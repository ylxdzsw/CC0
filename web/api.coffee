do ->
    await wasm_init

    window.INVALID_POSITION = (new Uint8Array libcc0.memory.buffer, libcc0.INVALID_POSITION, 1)[0]

    # JS does not have finalizers. We uses WeakRef to track living games and destory the underlying pointers when they are gone
    # the reclaim function is run when we are going to allocate a new instance.
    game_ptr_refs = []
    reclaim_game_ptr = ->
        game_ptr_refs = game_ptr_refs.filter ({ ref, ptr }) ->
            if not ref.deref()?
                console.log 'reclaimed a game'
                libcc0.destroy_game ptr
                false
            else
                true

    window.Game = class
        constructor: (board_type="standard") ->
            switch board_type
                when "standard"
                    @ptr = do libcc0.new_standard_game
                    do reclaim_game_ptr
                    game_ptr_refs.push ref: (new WeakRef @), ptr: @ptr
                when "small"
                    @ptr = do libcc0.new_small_game
                    do reclaim_game_ptr
                    game_ptr_refs.push ref: (new WeakRef @), ptr: @ptr
                else # construct directly with ptr
                    @ptr = board_type
                    # the ptr is owned by another instance.

            @board_size = do @get_board_size
            @n_pieces = do @get_n_pieces

        get_board_size: ->
            libcc0.get_board_size @ptr

        get_n_pieces: ->
            libcc0.get_n_pieces @ptr

        do_move: (old_pos, new_pos) ->
            libcc0.do_move @ptr, old_pos, new_pos

        possible_moves_with_path: (piece) ->
            ptr_buffer_ptr = libcc0.alloc_memory 8  # 4 bytes for buffer pointer (wasm32 is always 32 bit), and 4 bytes for size

            libcc0.possible_moves_with_path @ptr, piece, ptr_buffer_ptr, ptr_buffer_ptr + 4

            buffer_ptr = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr, ptr_buffer_ptr + 4)[0]
            size = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr + 4, ptr_buffer_ptr + 8)[0]
            buffer = new Uint8Array libcc0.memory.buffer, buffer_ptr, Number size

            libcc0.free_memory ptr_buffer_ptr, 8
            libcc0.free_memory buffer_ptr, size

            buffer[i] for i in [0...@board_size]

        dump: ->
            ptr_buffer_ptr = libcc0.alloc_memory 8 # 4 bytes for buffer pointer (wasm32 is always 32 bit), and 4 bytes for size

            libcc0.dump @ptr, ptr_buffer_ptr, ptr_buffer_ptr + 4

            buffer_ptr = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr, ptr_buffer_ptr + 4)[0]
            size = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr + 4, ptr_buffer_ptr + 8)[0]
            buffer = new Uint8Array libcc0.memory.buffer, buffer_ptr, Number size

            n_pieces = buffer[0]
            current_player = buffer[1]
            first_players_pieces = (buffer[i+2] for i in [0...n_pieces])
            second_players_pieces = (buffer[i+2+n_pieces] for i in [0...n_pieces])

            libcc0.free_memory ptr_buffer_ptr, 8
            libcc0.free_memory buffer_ptr, size

            { current_player, first_players_pieces, second_players_pieces }

        # 1: first player won, 2: second player won, 3: tie, 0: unfinished.
        get_status: ->
            libcc0.get_status @ptr

    window.MCTS = class
        constructor: () ->

