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

            for piece in key[1..game.board_size+1]
                x[1 + piece] = 1

            for piece in key[game.board_size+1..]
                x[1 + game.board_size + piece] = 1

        x

    await new Promise (resolve) ->
        document.querySelector '#download-model-button'
            .addEventListener 'click', ->
                if not onnx?
                    @disabled = true
                    document.querySelector '#download-model-status'
                        .classList.remove 'hidden'
                    onnx = await ort.InferenceSession.create 'model.onnx'
                    document.querySelector '#download-model-status'
                        .innerHTML = 'Model loaded'
                    do resolve

    window.model = {
        score: (game, key = null) ->
            input = new ort.Tensor 'float32', encode_input(game, key), [1, 1 + 2 * game.board_size]
            output = await onnx.run encoded_state: input
            prediction = output.value.data[0]
            if prediction < 0
                0
            else if prediction > 1
                1
            else
                prediction
    }

    player_menu.add "Alphabeta + Model", class
        move: ->
            await sleep 0

            sess = cc0.alphabeta_poll app.game.ptr, app.get_alphabeta_depth(), 0
            await sleep 0

            while sess != 0
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score app.game, key] for key in keys)
                sess = cc0.greedy_poll app.game.ptr, app.get_alphabeta_depth(), sess
                await sleep 0

            action = do read_wasm_json
            [action.from, action.to]

    player_menu.add "Greedy + Model", class
        move: ->
            await sleep 0

            sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), 0
            await sleep 0

            while sess != 0
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score app.game, key] for key in keys)
                sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), sess
                await sleep 0

            action = do read_wasm_json
            [action.from, action.to]

