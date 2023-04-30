do ->
    sess = null

    encode_game = (game) ->
        x = if game.is_p1_moving_next() then [0] else [1]
        for piece in game.p1_pieces()
            x.push piece + 2
        for piece in game.p2_pieces()
            x.push piece + 2 + game.board_size
        x

    encode_child = (game, child_pieces) ->
        x = if game.is_p1_moving_next() then [1] else [0]
        for piece in child_pieces[..game.n_pieces]
            x.push piece + 2
        for piece in child_pieces[game.n_pieces..]
            x.push piece + 2 + game.board_size
        x

    await new Promise (resolve) ->
        document.querySelector '#download-model-button'
            .addEventListener 'click', ->
                if not sess?
                    @disabled = true
                    document.querySelector '#download-model-status'
                        .classList.remove 'hidden'
                    sess = await ort.InferenceSession.create 'model.onnx'
                    window.sess = sess
                    document.querySelector '#download-model-status'
                        .innerHTML = 'Model loaded'
                    do resolve

    window.model = {
        score_game: (game) ->
            input = new ort.Tensor 'int32', encode_game(game), [1, 2 * game.n_pieces + 1]
            output = await sess.run encoded_state: input
            output.value.data[0]

        score_child: (game, child_pieces) ->
            input = new ort.Tensor 'int32', encode_child(game, child_pieces), [1, 2 * game.n_pieces + 1]
            output = await sess.run encoded_state: input
            output.value.data[0]
    }

    # player_menu.add "Transformer 18M", class
    #     move: ->
    #         tree = new MCTS
    #         { cont_ptr, game } = tree.start_try_playout app.game, do app.get_mcts_iter

    #         while cont_ptr
    #             [pieces, mask] = encode_input game
    #             pieces_tensor = new ort.Tensor 'int32', pieces, [1, 2 * game.n_pieces]
    #             mask_tensor = new ort.Tensor 'int32', mask, [1, game.n_pieces * game.board_size]
    #             { action_probs, value } = await sess.run { pieces: pieces_tensor, mask: mask_tensor }
    #             { cont_ptr, game } = tree.continue_try_playout cont_ptr, game, action_probs.data, value.data[0]
    #             await sleep 0

    #         tree.sample_action 0, 0.001
