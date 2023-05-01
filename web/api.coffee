do ->
    await wasm_ready

    window.INVALID_POSITION = (new Uint8Array cc0.memory.buffer, cc0.INVALID_POSITION, 1)[0]

    cc0.set_random_seed Math.floor Math.random() * (1 << 30)

    # reads the json buffer and returns the parsed json
    window.read_wasm_json = (parse = true) ->
        [ptr, len] = new Uint32Array cc0.memory.buffer, cc0.JSON_BUFFER, 2
        str = new TextDecoder().decode new Uint8Array cc0.memory.buffer, ptr, len
        do cc0.free_json_buffer
        if parse
            JSON.parse str
        else
            str

    # write to the json buffer. An API must be used to let the engine read and free the buffer
    window.write_wasm_json = (e, stringify = true) ->
        e = JSON.stringify e if stringify
        encoded = new TextEncoder().encode e
        cc0.alloc_json_buffer encoded.length
        buffer = new Uint32Array cc0.memory.buffer, cc0.JSON_BUFFER, 2
        new Uint8Array(cc0.memory.buffer, buffer[0], encoded.length).set(encoded)
        buffer[1] = encoded.length

    window.Game = class
        constructor: (board_type) ->
            switch board_type
                when "tiny"
                    @ptr = do cc0.new_tiny_game
                when "small"
                    @ptr = do cc0.new_small_game
                when "standard"
                    @ptr = do cc0.new_standard_game
                when "large"
                    @ptr = do cc0.new_large_game
                when "huge"
                    @ptr = do cc0.new_huge_game
                when "tiny+"
                    @ptr = do cc0.new_tiny_plus_game
                when "small+"
                    @ptr = do cc0.new_small_plus_game
                when "standard+"
                    @ptr = do cc0.new_standard_plus_game
                when "large+"
                    @ptr = do cc0.new_large_plus_game
                when "huge+"
                    @ptr = do cc0.new_huge_plus_game
                else
                    throw 0

            Object.assign @, @board_info()

        board_info: ->
            cc0.game_board_info @ptr
            do read_wasm_json

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

        turn: ->
            cc0.game_turn @ptr

        update_status_bar: ->
            app.update_status_bar "Turn": @turn()
            if @get_status() == 0
                app.update_status_bar "Status": if @is_p1_moving_next() then "Player 1 moving" else "Player 2 moving"
            else
                app.update_status_bar "Status": switch @get_status()
                    when 1 then "Player 1 won"
                    when 2 then "Player 2 won"
                    when 3 then "Tie"

            if window.model
                score = await window.model.score @
                app.update_status_bar "Model Estimation": (100 * score).toFixed 2

        free: ->
            cc0.free_game @ptr

    player_menu.add "Alphabeta + Heuristic", class
        move: ->
            await sleep 0
            cc0.alphabeta app.game.ptr, do app.get_alphabeta_depth
            await sleep 0
            action = do read_wasm_json
            [action.from, action.to]

    player_menu.add "Greedy + Heuristic", class
        move: ->
            await sleep 0
            cc0.greedy app.game.ptr, do app.get_temperature
            await sleep 0
            action = do read_wasm_json
            [action.from, action.to]

