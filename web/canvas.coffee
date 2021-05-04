window.canvas =
    init: (@board=StandardBoard, @scale=4, @padding=10) ->
        @svg = SVG().addTo('#canvas').size(100*@scale+2*@padding, 100*@scale+2*@padding)
        @slots = do @draw_board_skeleton

    draw_board_skeleton: ->
        @slot_group = do @svg.group
        @slots = for [x, y], i in @board.cartesian
            @slot_group.circle 15
                .center x*@scale+@padding, y*@scale+@padding
                .fill 'transparent'
                .stroke 'black'
                .remember 'id', i
                .on 'click', ->
                    console.log @remember 'id'

    move_no_trace: (old_pos, new_pos) ->
        color = @slots[old_pos].remember 'color'
        @slots[old_pos]
            .fill 'transparent'
            .forget 'color'
        @slots[new_pos]
            .fill color
            .remember 'color', color

    reset: ->
        [self_slots, oppo_slots] = @board.base_ids
        for slot in @slots
            switch
                when slot.remember('id') in self_slots
                    slot.fill 'blue'
                    slot.remember 'color', 'blue'
                when slot.remember('id') in oppo_slots
                    slot.fill 'red'
                    slot.remember 'color', 'red'
                else
                    slot.fill 'transparent'
