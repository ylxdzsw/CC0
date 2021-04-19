import ctypes

libcc0 = ctypes.cdll.LoadLibrary("../target/release/libcc0.so")

INVALID_POSITION = ctypes.c_uint8.in_dll(libcc0, "INVALID_POSITION").value

libcc0.alloc_memory.argtypes = [ctypes.c_uint64]
libcc0.alloc_memory.restype = ctypes.POINTER(ctypes.c_uint8)

libcc0.free_memory.argtypes = [ctypes.POINTER(ctypes.c_uint8), ctypes.c_uint64]
libcc0.free_memory.restype = None

libcc0.new_standard_game.argtypes = []
libcc0.new_standard_game.restype = ctypes.c_void_p

libcc0.new_small_game.argtypes = []
libcc0.new_small_game.restype = ctypes.c_void_p

libcc0.get_board_size.argtypes = [ctypes.c_void_p]
libcc0.get_board_size.restype = ctypes.c_uint64

libcc0.get_n_pieces.argtypes = [ctypes.c_void_p]
libcc0.get_n_pieces.restype = ctypes.c_uint64

libcc0.get_possible_moves.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.POINTER(ctypes.c_uint8)), ctypes.POINTER(ctypes.c_uint64)]
libcc0.get_possible_moves.restype = None

libcc0.do_move.argtypes = [ctypes.c_void_p, ctypes.c_uint8, ctypes.c_uint8]
libcc0.do_move.restype = None

libcc0.get_status.argtypes = [ctypes.c_void_p]
libcc0.get_status.restype = ctypes.c_uint8

libcc0.dump.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.POINTER(ctypes.c_uint8)), ctypes.POINTER(ctypes.c_uint64)]
libcc0.dump.restype = None

class Game:
    def __init__(self, board_type="standard"):
        if board_type == "standard":
            self.ptr = libcc0.new_standard_game()
        elif board_type == "small":
            self.ptr = libcc0.new_small_game()

    def get_board_size(self):
        return libcc0.get_board_size(self.ptr)

    def get_n_pieces(self):
        return libcc0.get_n_pieces(self.ptr)

    def get_possible_moves(self):
        buffer_ptr = ctypes.POINTER(ctypes.c_uint8)()
        size = ctypes.c_uint64(0)
        libcc0.get_possible_moves(self.ptr, ctypes.byref(buffer_ptr), ctypes.byref(size))

        possible_moves = []
        state = -1
        for i in range(size.value):
            x = buffer_ptr[i]
            if state == 0: # reading piece position
                possible_moves.append((x, []))
                state = 1
            else: # reading moving targets
                if x == INVALID_POSITION: # start next
                    state = 0
                    continue
                possible_moves[-1][1].append(x)

        libcc0.free_memory(buffer_ptr, size)

        return possible_moves

    def do_move(self, old_pos, new_pos):
        libcc0.do_move(self.ptr, old_pos, new_pos)

    # 1: first player won, 2: second player won, 3: tie, 0: unfinished.
    def get_status(self):
        return libcc0.get_status(self.ptr)

    # return (self pieces, opponent pieces)
    def dump(self):
        buffer_ptr = ctypes.POINTER(ctypes.c_uint8)()
        size = ctypes.c_uint64(0)
        libcc0.dump(self.ptr, ctypes.byref(buffer_ptr), ctypes.byref(size))

        n_pieces = buffer_ptr[0]
        current_player = buffer_ptr[1]
        first_players_pieces = [ buffer_ptr[i+2] for i in range(n_pieces) ]
        second_players_pieces = [ buffer_ptr[i+2+n_pieces] for i in range(n_pieces) ]

        libcc0.free_memory(buffer_ptr, size)

        if current_player == 1:
            return first_players_pieces, second_players_pieces
        elif current_player == 2:
            return second_players_pieces, first_players_pieces
