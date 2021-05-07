window.app =
    init: ->
        @game = new Game 'small'
        canvas.init 'small'
        do canvas.reset

    click: (pos) ->
        if @picked
            [old_pos, path] = @picked
            @game.do_move old_pos, pos
            do canvas.clear_all_highlighting
            canvas.move_no_trace old_pos, pos
            canvas.draw_path old_pos, pos, path

            @picked = null
            return

        do canvas.clear_all_path

        path = @game.possible_moves_with_path pos
        for next_hop, p in path
            if next_hop isnt INVALID_POSITION and p isnt pos
                canvas.highlight_slot p

        @picked = [pos, path]

    mouseover: (pos) ->

    mouseout: (pos) ->

    remote_move: (old_pos, new_pos) ->

window.replay = (records) ->
    button = document.createElement 'button'
    button.textContent = 'next'
    button.addEventListener 'click', ->
        [old_pos, new_pos] = do records.shift
        canvas.move_no_trace old_pos, new_pos

    document.querySelector 'body'
        .appendChild button

do ->
    await Promise.all window.ready

    do app.init
