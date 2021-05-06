window.ready.push do ->
    libcc0 = await do ->
        bytes = atob window.cc0_base64
        buffer = Uint8Array.from bytes, (c) -> c.charCodeAt 0
        WebAssembly.instantiate buffer, {}
            .then (x) => x.instance.exports

    # JS does not have finalizers. We uses WeakRef to track living games and destory the underlying pointers when they are gone
    # the reclaim function is run when we are going to allocate a new instance.
    game_ptr_refs = []
    reclaim_game_ptr = ->
        game_ptr_refs = game_ptr_refs.filter ({ ref, ptr }) ->
            if not ref.deref()?
                libcc0.destroy_game(ptr)
                false
            else
                true

    window.Game = class
        constructor: (board_type="standard") ->
            switch board_type
                when "standard"
                    @ptr = libcc0.new_standard_game()
                    do reclaim_game_ptr
                    game_ptr_refs.push({ ref: WeakRef(this), ptr: @ptr })
                when "small"
                    @ptr = libcc0.new_small_game()
                    do reclaim_game_ptr
                    game_ptr_refs.push({ ref: WeakRef(this), ptr: @ptr })
                else # construct directly with ptr
                    @ptr = board_type
                    # the ptr is owned by another instance.

            @board_size = @get_board_size()
            @n_pieces = @get_n_pieces()

        get_board_size: ->
            libcc0.get_board_size(@ptr)

        get_n_pieces: ->
            libcc0.get_n_pieces(@ptr)

        do_move: (old_pos, new_pos) ->
            libcc0.do_move(@ptr, old_pos, new_pos)

        # 1: first player won, 2: second player won, 3: tie, 0: unfinished.
        get_status: ->
            libcc0.get_status(@ptr)

    window.MCTS = class
        constructor: () ->

    # m = solver.alloc_memory 4n
    # m = new Uint8Array solver.memory.buffer, m, 4
    # m.set [15, 15, 15, 15]
    # r = solver.algorithm_x m.byteOffset, 4n
    # r = new Uint8Array solver.memory.buffer, r, 3
    # console.log r
    # solver.free_memory(m.byteOffset, 4n)
    # solver.free_memory(r.byteOffset, 3n)
