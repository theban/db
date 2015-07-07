require 'ffi'

class DBWrapper
  def initialize(ptr)
    @ptr = ptr
  end

  def get_db_pointer
    @ptr
  end
end

module RustTest
  extend FFI::Library
  ffi_lib 'libtag_db.so'
  attach_function :hallo_rust, :hallo_rust, [], :string
  attach_function :new_db, :new_db, [], :pointer
  attach_function :delete_db, :delete_db, [:pointer], :void
  attach_function :insert_db_intern, :insert_db, [:pointer, :string, :uint64, :uint64, :uint64, :pointer], :void

  def self.insert_db(ptr, name, range, data)
    raise unless ptr.is_a? DBWrapper
    memBuf = FFI::MemoryPointer.new(:char, data.size) # Create a memory pointer sized to the data
    memBuf.put_bytes(0, data)                         # Insert the actual data 
    insert_db_intern(ptr.get_db_pointer, name, range.min, range.max, data.size, memBuf)
  end
end

puts RustTest.hallo_rust()
db = RustTest.new_db()
RustTest.insert_db(DBWrapper.new(db), "fnord", 4..8, "hallo\0welt" )
RustTest.delete_db(db)
