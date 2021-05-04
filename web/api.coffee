window.api =
    init: ->
        bytes = atob window.cc0_base64
        buffer = Uint8Array.from bytes, (c) -> c.charCodeAt 0
        WebAssembly.instantiate buffer, {}
            .then (x) => @libcc0 = x.instance.exports

    # m = solver.alloc_memory 4n
    # m = new Uint8Array solver.memory.buffer, m, 4
    # m.set [15, 15, 15, 15]
    # r = solver.algorithm_x m.byteOffset, 4n
    # r = new Uint8Array solver.memory.buffer, r, 3
    # console.log r
    # solver.free_memory(m.byteOffset, 4n)
    # solver.free_memory(r.byteOffset, 3n)
