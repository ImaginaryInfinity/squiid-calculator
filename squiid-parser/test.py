import ctypes

# Load the Rust shared object file
rust_lib = ctypes.CDLL("./target/release/libsquiid_parser.so")

# Define the return type and argument types of get_strings function
rust_lib.parse_exposed.restype = ctypes.POINTER(ctypes.c_char_p)
rust_lib.parse_exposed.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_int)]

# Define the argument types of free_string_array function
rust_lib.free_string_array.argtypes = [ctypes.POINTER(ctypes.c_char_p), ctypes.c_int]

if __name__ == '__main__':
	# Call get_strings function and get the returned string array and length
	len_ptr = ctypes.c_int(0)
	s = rust_lib.parse_exposed(ctypes.create_string_buffer(b'sin(3+3(9))*8'), ctypes.byref(len_ptr))
	len_s = len_ptr.value

	# Print the strings
	mylist = [s[i].decode() for i in range(len_s)]
	print(mylist)

	rust_lib.free_string_array(s, len_s)