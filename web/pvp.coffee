peer = new Peer('cc0_283_ylxdzsw')
peer.on('open', function(id) {
  console.log('My peer ID is: ' + id);
})
peer.on('connection', function(conn) {
    conn.on('open', function() {
        conn.send('greetings from 1')

    })
    conn.on('data', function(data) {
        console.log('Received', data);
    })
    conn.on('error', function(err) {
        console.error(err)
    })
});
peer.on('error', function(err) {
    console.error(err)
})

peer = new Peer('cc0_33_ylxdzsw')
peer.on('open', function(id) {
  console.log('My peer ID is: ' + id);
  conn = peer.connect('cc0_283_ylxdzsw')
  conn.on('open', function(what) {
    conn.on('data', function(data) {
        console.log('Received:', data);
    })
    conn.on('error', function(err) {
        console.error(err)
    })
    conn.send('greetings from 2')
    conn.on('error', function(err) {
        console.error(err)
    })
  })

})
peer.on('error', function(err) {
    console.error(err)
})
