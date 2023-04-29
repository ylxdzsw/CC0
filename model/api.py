import json
import ctypes

libcc0 = ctypes.cdll.LoadLibrary("../target/release/libcc0.so")

INVALID_POSITION = ctypes.c_uint8.in_dll(libcc0, "INVALID_POSITION").value

JSON_BUFFER = (ctypes.c_size_t * 3).in_dll(libcc0, "JSON_BUFFER")

libcc0.alloc_json_buffer.argtypes = [ctypes.c_uint32]
libcc0.alloc_json_buffer.restype = None

libcc0.free_json_buffer.argtypes = []
libcc0.free_json_buffer.restype = None

libcc0.set_random_seed.argtypes = [ctypes.c_uint32]
libcc0.set_random_seed.restype = None

libcc0.new_small_game.argtypes = []
libcc0.new_small_game.restype = ctypes.c_void_p

libcc0.new_standard_game.argtypes = []
libcc0.new_standard_game.restype = ctypes.c_void_p

libcc0.free_game.argtypes = [ctypes.c_void_p]
libcc0.free_game.restype = None

libcc0.game_board_info.argtypes = [ctypes.c_void_p]
libcc0.game_board_info.restype = None

libcc0.game_is_p1_moving_next.argtypes = [ctypes.c_void_p]
libcc0.game_is_p1_moving_next.restype = ctypes.c_bool

libcc0.game_is_p2_moving_next.argtypes = [ctypes.c_void_p]
libcc0.game_is_p2_moving_next.restype = ctypes.c_bool

libcc0.game_p1_pieces.argtypes = [ctypes.c_void_p]
libcc0.game_p1_pieces.restype = None

libcc0.game_p2_pieces.argtypes = [ctypes.c_void_p]
libcc0.game_p2_pieces.restype = None

libcc0.game_get_status.argtypes = [ctypes.c_void_p]
libcc0.game_get_status.restype = ctypes.c_uint8

libcc0.game_move_to.argtypes = [ctypes.c_void_p, ctypes.c_uint8, ctypes.c_uint8]
libcc0.game_move_to.restype = None

libcc0.game_possible_moves_with_path.argtypes = [ctypes.c_void_p, ctypes.c_uint8]
libcc0.game_possible_moves_with_path.restype = None

libcc0.game_turn.argtypes = [ctypes.c_void_p]
libcc0.game_turn.restype = ctypes.c_size_t

libcc0.game_expand.argtypes = [ctypes.c_void_p]
libcc0.game_expand.restype = None

libcc0.alphabeta.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
libcc0.alphabeta.restype = None

libcc0.greedy.argtypes = [ctypes.c_void_p, ctypes.c_double]
libcc0.greedy.restype = None

def read_wasm_json():
    [ptr, size, _] = JSON_BUFFER
    s = ctypes.string_at(ptr, size)
    libcc0.free_json_buffer()
    return json.loads(s)

def write_wasm_json(obj):
    s = json.dumps(obj).encode("utf-8")
    libcc0.alloc_json_buffer(ctypes.c_uint32(len(s)))
    [ptr, *_] = JSON_BUFFER
    ctypes.memmove(ptr, s, len(s))
    JSON_BUFFER[1] = ctypes.c_size_t(len(s))

class Game:
    def __init__(self, board_type="standard"):
        match board_type:
            case "standard":
                self.ptr = libcc0.new_standard_game()
            case "small":
                self.ptr = libcc0.new_small_game()
            case _:
                raise ValueError(f"Unknown board type: {board_type}")

        self.n_pieces = self.board_info()["n_pieces"]
        self.board_size = self.board_info()["board_size"]

    def board_info(self):
        libcc0.game_board_info(self.ptr)
        return read_wasm_json()

    def is_p1_moving_next(self):
        return libcc0.game_is_p1_moving_next(self.ptr)

    def is_p2_moving_next(self):
        return libcc0.game_is_p2_moving_next(self.ptr)

    def p1_pieces(self):
        libcc0.game_p1_pieces(self.ptr)
        return read_wasm_json()

    def p2_pieces(self):
        libcc0.game_p2_pieces(self.ptr)
        return read_wasm_json()

    def get_status(self):
        return libcc0.game_get_status(self.ptr)

    def move_to(self, from_pos, to_pos):
        libcc0.game_move_to(self.ptr, from_pos, to_pos)

    def possible_moves_with_path(self, piece):
        libcc0.game_possible_moves_with_path(self.ptr, piece)
        return read_wasm_json()

    def turn(self):
        return libcc0.game_turn(self.ptr)

    def expand(self):
        libcc0.game_expand(self.ptr)
        return read_wasm_json()

    def __del__(self):
        libcc0.free_game(self.ptr)

def set_random_seed(seed):
    libcc0.set_random_seed(seed)

def alphabeta(game, depth):
    libcc0.alphabeta(game.ptr, depth)
    action = read_wasm_json()
    return [action["from"], action["to"]]

def greedy(game, temperature):
    libcc0.greedy(game.ptr, temperature)
    action = read_wasm_json()
    return [action["from"], action["to"]]
