do ->
    await wasm_ready

    window.INVALID_POSITION = (new Uint8Array cc0.memory.buffer, cc0.INVALID_POSITION, 1)[0]

    cc0.set_random_seed Math.floor Math.random() * (1 << 30)

    # reads the json buffer and returns the parsed json
    read_wasm_json = (parse = true) ->
        [ptr, len] = new Uint32Array cc0.memory.buffer, cc0.JSON_BUFFER, 2
        str = new TextDecoder().decode new Uint8Array cc0.memory.buffer, ptr, len
        do cc0.free_json_buffer
        if parse
            JSON.parse str
        else
            str

    # write to the json buffer. An API must be used to let the engine read and free the buffer
    write_wasm_json = (e, stringify = true) ->
        e = JSON.stringify e if stringify
        encoded = new TextEncoder().encode e
        cc0.alloc_json_buffer encoded.length
        buffer = new Uint32Array cc0.memory.buffer, cc0.JSON_BUFFER, 2
        new Uint8Array(cc0.memory.buffer, buffer[0], encoded.length).set(encoded)
        buffer[1] = encoded.length

    window.Game = class
        constructor: (board_type) ->
            switch board_type
                when "standard"
                    @ptr = do cc0.new_standard_game
                when "small"
                    @ptr = do cc0.new_small_game
                else
                    throw 0

        possible_moves_with_path: (pos) ->
            cc0.game_possible_moves_with_path @ptr, pos
            do read_wasm_json

        is_p1_moving_next: ->
            cc0.game_is_p1_moving_next @ptr

        is_p2_moving_next: ->
            cc0.game_is_p2_moving_next @ptr

        p1_pieces: ->
            cc0.game_p1_pieces @ptr
            do read_wasm_json

        p2_pieces: ->
            cc0.game_p2_pieces @ptr
            do read_wasm_json

        # 0: pending, 1: p1 won, 2: p2 won, 3: tie
        get_status: ->
            cc0.game_get_status @ptr

        move_to: (old_pos, new_pos) ->
            cc0.game_move_to @ptr, old_pos, new_pos


    player_menu.add "Alphabeta Player", class
        move: ->
            cc0.alphabeta app.game.ptr, do app.get_alphabeta_depth
            action = do read_wasm_json
            await sleep 0
            [action.from, action.to]

