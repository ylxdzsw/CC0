canvas =
    init: (@board=StandardBoard, @scale=4, @padding=10) ->
        @svg = SVG().addTo('body').size(100*@scale+2*@padding, 100*@scale+2*@padding)
        @slots = do @draw_board_skeleton

    draw_board_skeleton: ->
        @slot_group = do @svg.group
        @slots = for [x, y], i in @board.cartesian
            @slot_group.circle 15
                .center x*@scale+@padding, y*@scale+@padding
                .fill 'transparent'
                .stroke 'black'
                .remember 'id', i

    reset: ->
        [self_slots, oppo_slots] = @board.base_ids
        for slot in @slots
            switch
                when slot.remember('id') in self_slots
                    slot.fill 'blue'
                when slot.remember('id') in oppo_slots
                    slot.fill 'red'
                else
                    slot.fill 'transparent'


main = (ready) ->
    await ready
    canvas.init StandardBoard

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

main new Promise (res) -> window.wasm_ready = res
