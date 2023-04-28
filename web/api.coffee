do ->
    await wasm_init

    window.INVALID_POSITION = (new Uint8Array libcc0.memory.buffer, libcc0.INVALID_POSITION, 1)[0]

    libcc0.set_random_seed Math.floor Math.random() * (1 << 30)

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
            buffer = new Uint8Array libcc0.memory.buffer, buffer_ptr, size

            result = (buffer[i] for i in [0...@board_size])

            libcc0.free_memory ptr_buffer_ptr, 8
            libcc0.free_memory buffer_ptr, size

            result

        all_possible_moves: ->
            ptr_buffer_ptr = libcc0.alloc_memory 8

            libcc0.all_possible_moves @ptr, ptr_buffer_ptr, ptr_buffer_ptr + 4

            buffer_ptr = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr, ptr_buffer_ptr + 4)[0]
            size = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr + 4, ptr_buffer_ptr + 8)[0]
            buffer = new Uint8Array libcc0.memory.buffer, buffer_ptr, size

            possible_moves = []
            state = -1
            for i in [0...size]
                x = buffer[i]
                if state is 0
                    break if x is INVALID_POSITION
                    possible_moves.push [x, []]
                    state = 1
                else
                    if x is INVALID_POSITION
                        state = 0
                        continue
                    possible_moves[possible_moves.length-1][1].push x

            libcc0.free_memory ptr_buffer_ptr, 8
            libcc0.free_memory buffer_ptr, size

            possible_moves

        dump: ->
            ptr_buffer_ptr = libcc0.alloc_memory 8 # 4 bytes for buffer pointer (wasm32 is always 32 bit), and 4 bytes for size

            libcc0.dump @ptr, ptr_buffer_ptr, ptr_buffer_ptr + 4

            buffer_ptr = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr, ptr_buffer_ptr + 4)[0]
            size = (new Uint32Array libcc0.memory.buffer.slice ptr_buffer_ptr + 4, ptr_buffer_ptr + 8)[0]
            buffer = new Uint8Array libcc0.memory.buffer, buffer_ptr, size

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

    mcts_ptr_refs = []
    reclaim_mcts_ptr = ->
        mcts_ptr_refs = mcts_ptr_refs.filter ({ ref, ptr }) ->
            if not ref.deref()?
                console.log 'reclaimed a mcts'
                libcc0.destroy_mcts ptr
                false
            else
                true

    window.MCTS = class
        constructor: (heuristic = false) ->
            if heuristic
                @ptr = do libcc0.new_mcts_pure
            else
                @ptr = do libcc0.new_mcts_heuristic
            do reclaim_mcts_ptr
            mcts_ptr_refs.push ref: (new WeakRef @), ptr: @ptr

        playout: (game, ntimes) ->
            libcc0.mcts_playout @ptr, game.ptr, ntimes

        start_try_playout: (game, ntimes) ->
            game_ptr_ptr = libcc0.alloc_memory 4
            game_ptr_buf = new Uint32Array libcc0.memory.buffer, game_ptr_ptr, 1
            game_ptr_buf[0] = game.ptr

            cont_ptr = libcc0.start_try_playout @ptr, game_ptr_ptr, ntimes
            game_ptr = game_ptr_buf[0]
            libcc0.free_memory game_ptr_ptr, 4

            { cont_ptr, game: (if game_ptr then new Game game_ptr else null) }

        continue_try_playout: (cont_ptr, game, prior, value) ->
            ptr_buffer_ptr = libcc0.alloc_memory 8 # cont_ptr and game_ptr
            ptr_buffer = new Uint32Array libcc0.memory.buffer, ptr_buffer_ptr, 2
            ptr_buffer[0] = cont_ptr
            ptr_buffer[1] = game.ptr
            prior_buffer_ptr = libcc0.alloc_memory 4 * prior.length # todo: reuse these buffers?
            prior_buffer = new Float32Array libcc0.memory.buffer, prior_buffer_ptr, prior.length
            prior_buffer[i] = Math.exp i for p, i in prior
            libcc0.continue_try_playout ptr_buffer_ptr, ptr_buffer_ptr + 4, prior_buffer_ptr, value
            new_cont_ptr = ptr_buffer[0]
            new_game_ptr = ptr_buffer[1]
            libcc0.free_memory ptr_buffer_ptr, 8
            libcc0.free_memory prior_buffer_ptr, 4 * prior.length
            { cont_ptr: new_cont_ptr, game: (if new_game_ptr then new Game new_game_ptr else null) }

        sample_action: (exploration_prob, temperature) ->
            action = libcc0.mcts_sample_action @ptr, exploration_prob, temperature
            [action >> 8, action & 0xff]

        total_visits: ->
            libcc0.mcts_total_visits @ptr

        root_value: ->
            libcc0.mcts_root_value @ptr

    player_menu.add "Pure MCTS", class
        move: ->
            tree = new MCTS
            n_iter = 0
            while n_iter < do app.get_mcts_iter
                tree.playout app.game, 100
                n_iter += 100
                await sleep 0
            tree.sample_action 0, 0.001

    player_menu.add "Heuristic MCTS", class
        move: ->
            tree = new MCTS true
            n_iter = 0
            while n_iter < do app.get_mcts_iter
                tree.playout app.game, 100
                n_iter += 100
                await sleep 0
            tree.sample_action 0, 0.001
