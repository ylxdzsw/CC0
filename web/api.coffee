window.api =
    init: ->
        bytes = atob window.cc0_base64
        buffer = Uint8Array.from bytes, (c) -> c.charCodeAt 0
        WebAssembly.instantiate buffer, {}
            .then (x) => @libcc0 = x.instance.exports


do window.api.init

