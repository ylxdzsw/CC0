palette =
    blue_light: '#dae8fc'
    blue_solid: '#6c8ebf'
    green_light: '#d5e8d4'
    green_solid: '#82b366'
    red_light: '#f8cecc'
    red_solid: '#b85450'
    gray: '#ccc'

window.canvas =
    init: (board_type='standard', @scale=4, @padding=10) ->
        switch board_type
            when 'tiny' then @board = TinyBoard
            when 'small' then @board = SmallBoard
            when 'standard' then @board = StandardBoard
            when 'large' then @board = LargeBoard
            when 'huge' then @board = HugeBoard
            else return console.error 'unknown board'

        document.querySelector('#canvas').innerHTML = ''
        @svg = SVG().addTo('#canvas').size(100*@scale+2*@padding, 100*@scale+2*@padding)
        do @draw_board_skeleton

    draw_board_skeleton: ->
        handler_stub =
            click: (pos) => @handler?.click? pos
            mouseover: (pos) => @handler?.mouseover? pos
            mouseout: (pos) => @handler?.mouseout? pos
        @slot_group = do @svg.group
        @slots = for [x, y], i in @board.cartesian
            @slot_group.circle 20
                .center x*@scale+@padding, y*@scale+@padding
                .fill 'transparent'
                .stroke 'black'
                .remember 'id', i
                .on 'click', -> handler_stub.click @remember 'id'
                .on 'mouseover', -> handler_stub.mouseover @remember 'id'
                .on 'mouseout', -> handler_stub.mouseout @remember 'id'

    highlight_slot: (id) ->
        @slots[id]
            .fill palette.gray
            .remember 'color', palette.gray

    clear_all_highlighting: ->
        for slot in @slots
            if (slot.remember 'color') is palette.gray
                slot.fill 'transparent'
                slot.forget 'color'

    draw_path: (old_pos, new_pos, path) ->
        return if old_pos is new_pos # done

        next_hop = path[new_pos]
        [x1, y1] = @board.cartesian[next_hop]
        [x2, y2] = @board.cartesian[new_pos]
        @svg.line [[x1*@scale+@padding, y1*@scale+@padding], [x2*@scale+@padding, y2*@scale+@padding]]
            .addClass 'path'
            .stroke 'black'
            .attr 'stroke-dasharray', '2, 2'

        canvas.draw_path old_pos, next_hop, path

    clear_all_path: ->
        @svg.find 'line.path'
            .remove()

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
                    slot.fill palette.blue_solid
                    slot.remember 'color', palette.blue_solid
                when slot.remember('id') in oppo_slots
                    slot.fill palette.red_solid
                    slot.remember 'color', palette.red_solid
                else
                    slot.fill 'transparent'

    install_handler: (@handler) ->
