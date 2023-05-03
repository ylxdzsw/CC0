do ->
    onnx = null

    encode_input = (game, key = null) ->
        x = Array(1 + 2 * game.board_size).fill 0

        if key == null
            if game.is_p1_moving_next()
                x[0] = 1

            for piece in game.p1_pieces()
                x[1 + piece] = 1

            for piece in game.p2_pieces()
                x[1 + game.board_size + piece] = 1
        else
            if key[0] % 2 == 0
                x[0] = 1

            for piece in key[1...game.board_size+1]
                x[1 + piece] = 1

            for piece in key[game.board_size+1...]
                x[1 + game.board_size + piece] = 1

        x

    await new Promise (resolve) ->
        document.querySelector '#download-model-button'
            .addEventListener 'click', ->
                if not onnx?
                    @disabled = true
                    document.querySelector '#download-model-status'
                        .textContent = 'Downloading...'
                    onnx =
                        small: await ort.InferenceSession.create 'small.onnx'
                        # standard: await ort.InferenceSession.create 'standard.onnx'
                    document.querySelector '#download-model-status'
                        .textContent = 'Model loaded'
                    do resolve

    window.model =
        supports: (game) -> onnx[game.board_type]?
        score: (game, key = null) ->
            input = new ort.Tensor 'float32', encode_input(game, key), [1, 1 + 2 * game.board_size]
            output = await onnx[game.board_type].run encoded_state: input
            prediction = output.value.data[0]
            1 / (1 + Math.exp -prediction)
        score_expectation: (game) ->
            keys = game.expand()
            scores = for key in keys
                await window.model.score game, key
            softmax_expectation scores, 0.2, game.is_p2_moving_next()

    player_menu.add "Alphabeta + Model", ['small'], class
        move: ->
            await sleep 0

            sess = cc0.alphabeta_poll app.game.ptr, app.get_alphabeta_depth(), app.get_forward_only(), 0
            await sleep 0

            while sess != 0
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score app.game, key] for key in keys)
                sess = cc0.alphabeta_poll app.game.ptr, app.get_alphabeta_depth(), app.get_forward_only(), sess
                await sleep 0

            do read_wasm_json

    player_menu.add "Greedy + Model", ['small'], class
        move: ->
            await sleep 0

            sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), app.get_forward_only(), 0
            await sleep 0

            while sess != 0
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score app.game, key] for key in keys)
                sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), app.get_forward_only(), sess
                await sleep 0

            do read_wasm_json

    player_menu.add "MCTS + Model", ['small'], class
        move: ->
            await sleep 0

            sess = cc0.mcts_poll app.game.ptr, app.get_mcts_iter(), app.get_forward_only(), 0
            await sleep 0

            while sess != 0
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score app.game, key] for key in keys)
                sess = cc0.mcts_poll app.game.ptr, app.get_mcts_iter(), app.get_forward_only(), sess
                await sleep 0

            do read_wasm_json
