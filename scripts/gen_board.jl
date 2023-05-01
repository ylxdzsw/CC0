using OhMyJulia
using Fire

# x axis goes from the lower left corner to the horizonal right, starts from 0
# y axis goes from the lower left corner to the upper right, starts from 0
mutable struct Node
    id::Union{Int, Nothing}
    x::Int
    y::Int
end

Node(x::Int, y::Int) = Node(nothing, x, y)

cartesian(node::Node) = (round(node.x + cos(π/3) * node.y, digits=5), round(sin(π/3) * node.y, digits=5))

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
    sort!(nodes, by=reverse ∘ cartesian)
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

    open("board.svg", "w") do f
        write(f, strip(svg))
    end
end

# each row in the adj_matrix is the id of neibours of that node, in the order of UL, UR, R, LR, LL, L
function gen_adj_matrix(nodes)
    dict = Dict((n.x, n.y) => n.id for n in nodes)
    mat = Matrix{Int}(undef, length(nodes), 6)
    for n in nodes
        UL = get(dict, (n.x-1, n.y+1), 255)
        UR = get(dict, (n.x,   n.y+1), 255)
        R  = get(dict, (n.x+1, n.y), 255)
        LR = get(dict, (n.x+1, n.y-1), 255)
        LL = get(dict, (n.x,   n.y-1), 255)
        L  = get(dict, (n.x-1, n.y), 255)
        mat[n.id+1, :] .= UL, UR, R, LR, LL, L
    end
    mat
end

function write_adj_matrix(mat)
    buf = IOBuffer()
    buf << '['
    for i in 1:size(mat)[1]
        buf << "[$(join(mat[i, :], ','))],"
    end
    skip(buf, -1)
    buf << ']'
    take!(buf) |> String
end

function gen_base_ids(nodes, rank)
    self_base_ids = sort([node.id for node in nodes])[1:sum(1:rank)]
    oppo_base_ids = reverse(reverse(sort([node.id for node in nodes]))[1:sum(1:rank)])
    self_base_ids, oppo_base_ids
end

function gen_base_ids_plus(base_ids, adj_matrix)
    reachables = Set{Int}()
    for id in base_ids
        reachables = reachables ∪ Set(adj_matrix[id+1, :])
    end
    delete!(reachables, 255)
    reachables |> collect |> sort
end

@main function rust(rank::Int)
    nodes = gen_nodes(rank)
    label!(nodes)
    println(length(nodes))
    self_base_ids, oppo_base_ids = gen_base_ids(nodes, rank)
    println(length(self_base_ids))
    println(self_base_ids)
    println(oppo_base_ids)
    adj_matrix = gen_adj_matrix(nodes)
    println(write_adj_matrix(adj_matrix))
    println(score(nodes, adj_matrix, oppo_base_ids[end]))
    println(score(nodes, adj_matrix, self_base_ids[1]))
    x = score(nodes, adj_matrix, self_base_ids[1])
    println(sum(x[i+1] for i in self_base_ids))

    println(gen_base_ids_plus(self_base_ids, adj_matrix))
    println(gen_base_ids_plus(oppo_base_ids, adj_matrix))
    println(length(gen_base_ids_plus(self_base_ids, adj_matrix)))
    println(sum(x[i+1] for i in gen_base_ids_plus(self_base_ids, adj_matrix)))
end

# recenter in (100, 100) and make y axis from top to down
function svg_cartesian(positions)
    left_most = minimum(car.(positions))
    right_most = maximum(car.(positions))
    bottom = minimum(cadr.(positions))
    top = maximum(cadr.(positions))

    map(positions) do pos
        x, y = pos
        x = (x - left_most) / (right_most - left_most)
        y = 1 - (y - bottom) / (top - bottom)
        round(100x, digits=2), round(100y, digits=2)
    end
end

@main function web(rank::Int)
    nodes = gen_nodes(rank)
    label!(nodes)
    println(collect(gen_base_ids(nodes, rank)))
    println(collect.(svg_cartesian(cartesian.(nodes))))
end

# calculate the distance from each node to the target with BFS
function score(nodes, adj, target)
    dist = fill(255, length(nodes))
    dist[target+1] = 0
    queue = [target]
    while !isempty(queue)
        node = popfirst!(queue)
        for neibour in adj[node+1, :] @when neibour != 255 && dist[neibour+1] == 255
            dist[neibour+1] = dist[node+1] + 1
            push!(queue, neibour)
        end
    end
    dist
end
