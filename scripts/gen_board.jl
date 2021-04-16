using OhMyJulia

# x axis goes from the lower left corner to the horizonal right, starts from 0
# y axis goes from the lower left corner to the upper right, starts from 0
mutable struct Node
    id::Union{Int, Nothing}
    x::Int
    y::Int
end

Node(x::Int, y::Int) = Node(nothing, x, y)

cartesian(node::Node) = (node.x + cos(π/3) * node.y, sin(π/3) * node.y)

# generate the nodes and they positions
function gen_nodes(rank=4)
    nodes = Node[]

    # the main triangle including the lower left, top, and lower right corners
    for i in 0:3rank, j in 0:3rank @when i+j <= 3rank
        push!(nodes, Node(i, j))
    end

    # the upper left corner
    for i in -1:-1:-rank, j in rank+1:2rank @when i + j >= rank
        push!(nodes, Node(i, j))
    end

    # the bottom corner
    for i in rank+1:2rank, j in -1:-1:-rank @when i + j >= rank
        push!(nodes, Node(i, j))
    end

    # the upper right corner
    for i in rank+1:2rank, j in rank+1:2rank @when i+j >= 3rank+1
        push!(nodes, Node(i, j))
    end

    nodes
end

# label the nodes from top to bottom using cartesian axis
function label!(nodes::Vector{Node})
    nodes = sort(nodes, by=reverse ∘ cartesian)
    for (i, n) in enumerate(nodes)
        n.id = i-1
    end
end

function write_svg(nodes)
    list = String[]

    for node in nodes
        x, y = cartesian(node)
        x = round(Int, 20x)
        y = round(Int, 20y)
        push!(list, """
            <circle cx="$x" cy="$y" r="8" stroke="black" fill="transparent"/>
            <text x="$x" y="$(y+.5)" class="t" alignment-baseline="middle" text-anchor="middle">$(node.id)</text>
        """)
    end

    svg = """
        <?xml version="1.0" encoding="UTF-8"?>
        <svg viewBox="-10 -80 300 300" xmlns="http://www.w3.org/2000/svg" version="1.1">
            <style> .t { font: italic 6px sans-serif; } </style>
            $(join(list, '\n'))
        </svg>
    """

    open("output.svg", "w") do f
        write(f, strip(svg))
    end
end
