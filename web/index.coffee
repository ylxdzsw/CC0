window.player_menu = do ->
    class PlayerMenuItem
        @next_id: 0
        constructor: (@name, @agent) ->
            @id = PlayerMenuItem.next_id++

    items: []

    new: (id) ->
        item = @items.find (x) => x.id is id
        new item.agent

    add: (name, agent) ->
        item = new PlayerMenuItem name, agent
        @items.push item
        for i in [1..2]
            el = document.querySelector "#player-#{i}"
            option = document.createElement 'option'
            option.setAttribute 'value', "i#{item.id}"
            option.textContent = name
            el.appendChild option
            el.value = "i#{item.id}" if not el.value

    remove: (id) ->
        @items = @items.filter (x) -> x.id isnt id
        for i in [1..2]
            el = document.querySelector "#player-#{i}"
            el.value = "i#{@items[0].id}" if el.value is "i#{id}"
        document.querySelectorAll "option[value=\"i#{id}\"]"
            .remove()

player_menu.add "Local Player", class
    move: -> new Promise (resolve, reject) =>
        canvas.install_handler
            click: (pos) =>
                { role, path } = app.pos_info pos
                if @picked
                    switch role
                        when 0
                            [old_pos, old_path] = @picked
                            if old_path[pos] is INVALID_POSITION
                                return
                            do canvas.clear_all_highlighting
                            @picked = null
                            canvas.install_handler null
                            resolve [old_pos, pos]
                        when 1
                            @picked = [pos, path]
                            do canvas.clear_all_highlighting
                            canvas.highlight_slot p for next_hop, p in path when next_hop isnt INVALID_POSITION and p isnt pos
                else
                    if role isnt 1
                        return
                    do canvas.clear_all_path
                    do canvas.clear_all_highlighting
                    canvas.highlight_slot p for next_hop, p in path when next_hop isnt INVALID_POSITION and p isnt pos
                    @picked = [pos, path]
            mouseover: (pos) =>
                # console.log app.pos_info pos
            mouseout: (pos) =>

window.app =
    init: ->
        document.querySelector '#mctc-iter-slider'
            .addEventListener 'change', =>
                document.querySelector('#mctc-iter-slider-label-num').textContent = do @get_mcts_iter

        document.querySelector '#alphabeta-depth-slider'
            .addEventListener 'change', =>
                document.querySelector('#alphabeta-depth-slider-label-num').textContent = do @get_alphabeta_depth

        document.querySelector '#new-game-button'
            .addEventListener 'click', => do @new_game

    get_mcts_iter: ->
        v = Number document.querySelector('#mctc-iter-slider').value
        Math.floor v * v

    get_alphabeta_depth: ->
        Number document.querySelector('#alphabeta-depth-slider').value

    # role: 0: empty, 1: self piece, 2: opponenet piece
    pos_info: (pos) ->
        role = switch
            when @game.is_p1_moving_next() and pos in @game.p1_pieces() or @game.is_p2_moving_next() and pos in @game.p2_pieces() then 1
            when @game.is_p1_moving_next() and pos in @game.p2_pieces() or @game.is_p2_moving_next() and pos in @game.p1_pieces() then 2
            else 0
        if role
            path = @game.possible_moves_with_path pos
            { role, path }
        else
            { role }

    new_game: ->
        board_type = document.querySelector('#board-type').value ? 'standard'
        @game = new Game board_type

        canvas.init board_type
        do canvas.reset

        player1 = player_menu.new parseInt document.querySelector("#player-1").value.slice(1)
        player2 = player_menu.new parseInt document.querySelector("#player-2").value.slice(1)

        loop
            current_player = if @game.is_p1_moving_next() then player1 else player2
            [old_pos, new_pos] = await do current_player.move
            { path } = @pos_info old_pos

            @game.move_to old_pos, new_pos

            do canvas.clear_all_path
            do canvas.clear_all_highlighting
            canvas.move_no_trace old_pos, new_pos
            canvas.draw_path old_pos, new_pos, path

            switch do @game.get_status
                when 1 then return @end_game 'player 1 won'
                when 2 then return @end_game 'player 2 won'
                when 3 then return @end_game 'tie'

    end_game: (msg) ->
        console.log msg


window.replay = (records) ->
    button = document.createElement 'button'
    button.textContent = 'next'
    button.addEventListener 'click', ->
        [old_pos, new_pos, path] = do records.shift
        canvas.move_no_trace old_pos, new_pos
        canvas.draw_path old_pos, new_pos, path

    document.querySelector 'body'
        .appendChild button

window.sleep = (ms) -> new Promise (resolve) -> setTimeout resolve, ms

do ->
    await wasm_ready
    await sleep 0 # allow other components to initialize first

    do app.init
