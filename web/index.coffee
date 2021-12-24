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
                console.log pos
                if @picked
                    [old_pos, path] = @picked
                    do canvas.clear_all_highlighting
                    @picked = null
                    resolve [old_pos, pos, path]
                else
                    do canvas.clear_all_path
                    path = app.game.possible_moves_with_path pos
                    canvas.highlight_slot p for next_hop, p in path when next_hop isnt INVALID_POSITION and p isnt pos
                    @picked = [pos, path]
            mouseover: (pos) =>
            mouseout: (pos) =>

window.app =
    init: ->
        document.querySelector '#new-game-button'
            .addEventListener 'click', => do @new_game

    new_game: ->
        @game = new Game 'standard'
        canvas.init 'standard'
        do canvas.reset

        player1 = player_menu.new parseInt document.querySelector("#player-1").value.slice(1)
        player2 = player_menu.new parseInt document.querySelector("#player-2").value.slice(1)

        n_moves = 0
        loop
            current_player = [player1, player2][n_moves % 2]
            [old_pos, new_pos, path] = await do current_player.move

            @game.do_move old_pos, new_pos
            do canvas.clear_all_path
            do canvas.clear_all_highlighting
            canvas.move_no_trace old_pos, new_pos
            canvas.draw_path old_pos, new_pos, path

            switch do @game.get_status
                when 1 then return @end_game 'player 1 won'
                when 2 then return @end_game 'player 1 won'
                when 3 then return @end_game 'tie'

    end_game: (msg) ->
        console.log msg


window.replay = (records) ->
    button = document.createElement 'button'
    button.textContent = 'next'
    button.addEventListener 'click', ->
        [old_pos, new_pos] = do records.shift
        canvas.move_no_trace old_pos, new_pos

    document.querySelector 'body'
        .appendChild button

do ->
    await wasm_init
    await new Promise (resolve) -> setTimeout resolve # allow other components to initialize first

    do app.init
