do ->
    onnx = null

    encode_game = (game) ->
        x = Array(1 + 2 * game.board_size).fill 0

        if game.is_p1_moving_next()
            x[0] = 1

        for piece in game.p1_pieces()
            x[1 + piece] = 1

        for piece in game.p2_pieces()
            x[1 + game.board_size + piece] = 1

        x

    encode_child = (game, child_pieces) ->
        x = Array(1 + 2 * game.board_size).fill 0

        if game.is_p2_moving_next()
            x[0] = 1

        for piece in child_pieces[..game.n_pieces]
            x[1 + piece] = 1

        for piece in child_pieces[game.n_pieces..]
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
        score_game: (game) ->
            input = new ort.Tensor 'float32', encode_game(game), [1, 1 + 2 * game.board_size]
            output = await onnx.run encoded_state: input
            output.value.data[0]

        score_child: (game, child_pieces) ->
            input = new ort.Tensor 'float32', encode_child(game, child_pieces), [1, 1 + 2 * game.board_size]
            output = await onnx.run encoded_state: input
            output.value.data[0]
    }

    player_menu.add "Greedy + Model", class
        move: ->
            await sleep 0

            sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), 0
            await sleep 0

            while sess != 0
                console.log sess
                keys = do read_wasm_json
                write_wasm_json ([key, await window.model.score_child app.game, key[1..]] for key in keys)
                sess = cc0.greedy_poll app.game.ptr, app.get_temperature(), sess
                await sleep 0

            action = do read_wasm_json
            [action.from, action.to]
