window.app =
    init: (remote_player) ->

    click: (pos) ->

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
    await Promise.all [
        do api.init
        do pvp.init
    ]

    do canvas.init
    canvas.reset 'small'


