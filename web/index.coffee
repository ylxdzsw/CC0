app =
    init: ->



    select: (id) ->


window.replay = (records) ->
    button = document.createElement 'button'
    button.innerHTML = 'next'
    button.addEventListener 'click', ->
        [old_pos, new_pos] = do records.shift
        canvas.move_no_trace old_pos, new_pos

    document.querySelector 'body'
        .appendChild button

main = (ready) ->
    await ready
    canvas.init SmallBoard

    do canvas.reset


    # rect = draw.rect 100, 100
    #     .attr fill: '#f06'

    # m = solver.alloc_memory 4n
    # m = new Uint8Array solver.memory.buffer, m, 4
    # m.set [15, 15, 15, 15]
    # r = solver.algorithm_x m.byteOffset, 4n
    # r = new Uint8Array solver.memory.buffer, r, 3
    # console.log r
    # solver.free_memory(m.byteOffset, 4n)
    # solver.free_memory(r.byteOffset, 3n)

