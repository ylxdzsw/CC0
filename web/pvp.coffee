window.ready.push do ->
    self_id = Math.floor Math.random() * 1000000

    document.querySelector '#self-id'
        .textContent = "My id: #{@self_id}"

    conn = null

    peer = await new Promise (resolve, reject) ->
        new Peer "cc0_#{self_id}_cc0"
            .on 'open', -> do resolve
            .on 'error', (err) -> handle_peer_error err
            .on 'connection', (conn) -> accept_connection conn
            .on 'disconnected', ->
                console.error 'Peer disconnected'
                # do peer.reconnect

    connect = (target_id) -> new Promise (resolve, reject) =>
        if conn?
            console.error "try to connect to #{target_id} while connected"
            do reject

        conn = peer.connect "cc0_#{target_id}_cc0"
            .on 'open', -> do resolve # TODO: mark the connection as ready
            .on 'data', (msg) -> handle_message msg
            .on 'error', (err) -> handle_conn_error err

    accept_connection: (incoming) ->
        if conn?
            console.error "being connected while already connected"
            # todo: terminate the incoming connection

        conn = incoming
            .on 'open', -> # TODO: mark the connection as ready
            .on 'data', (msg) -> handle_message msg
            .on 'error', (err) -> handle_conn_error err

    handle_peer_error: (err) ->
        console.error err

    handle_conn_error: (err) ->
        console.error err

    handle_message: (msg) ->
        console.log 'received: ' + msg
