window.pvp =
    init: -> new Promise (resolve, reject) =>
        @self_id = Math.floor(Math.random() * 1000000)
        @peer = new Peer "cc0_#{@self_id}_cc0"
        @peer.on 'open', => do resolve
        @peer.on 'error', (err) => @handle_peer_error err
        @peer.on 'connection', (conn) => @accept_connection conn
        @status = 'listening'
        el = document.querySelector '#self-id'
        el.textContent = "My id: #{@self_id}"

    connect: (target_id) -> new Promise (resolve, reject) =>
        if @status isnt 'listening'
            console.error "try to connect to #{target_id} while #{@status}"
            do reject

        @conn = peer.connect "cc0_#{target_id}_cc0"
        @conn.on 'open', =>
            @status = 'connected'
            do resolve
        @conn.on 'data', (msg) => @handle_message msg
        @conn.on 'error', (err) => @handle_conn_error err

        @status = 'connecting'

    accept_connection: (conn) ->
        if @status isnt 'listening'
            console.error "being connected while #{@status}"
            # todo: terminate the incoming connection

        @conn = conn
        @conn.on 'open', =>
            @status = 'connected'
        @conn.on 'data', (msg) => @handle_message msg
        @conn.on 'error', (err) => @handle_conn_error err

        @status = 'connecting'

    handle_peer_error: (err) ->
        console.error err

    handle_conn_error: (err) ->
        console.error err

    handle_message: (msg) ->
        console.log 'received: ' + msg
