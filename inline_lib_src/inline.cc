
#include "inline.h"

#include "Recast.h"

rcContext* CreateContext(bool state) { return new rcContext(state); }

void DeleteContext(rcContext* context) { delete context; }

class ForwardVtableTileCacheMeshProcess : public dtTileCacheMeshProcess {
 public:
  ForwardVtableTileCacheMeshProcess(
      void* object_ptr, ForwardVtableTileCacheMeshProcessProcessFn process_fn)
      : object_ptr(object_ptr), process_fn(process_fn) {}

  void process(dtNavMeshCreateParams* params, unsigned char* polyAreas,
               unsigned short* polyFlags) override {
    process_fn(object_ptr, params, polyAreas, polyFlags);
  }

 private:
  void* object_ptr;
  ForwardVtableTileCacheMeshProcessProcessFn process_fn;
};

dtTileCacheMeshProcess* CreateForwardedTileCacheMeshProcess(
    void* object_ptr, ForwardVtableTileCacheMeshProcessProcessFn process_fn) {
  return new ForwardVtableTileCacheMeshProcess(object_ptr, process_fn);
}

void DeleteTileCacheMeshProcess(dtTileCacheMeshProcess* mesh_process) {
  delete mesh_process;
}

dtTileCacheAlloc* CreateDefaultTileCacheAlloc() {
  return new dtTileCacheAlloc();
}

void DeleteTileCacheAlloc(dtTileCacheAlloc* alloc) { delete alloc; }

class ForwardVtableTileCacheAlloc : public dtTileCacheAlloc {
 public:
  ForwardVtableTileCacheAlloc(void* object_ptr,
                              ForwardVtableTileCacheAllocResetFn reset_fn,
                              ForwardVtableTileCacheAllocAllocFn alloc_fn,
                              ForwardVtableTileCacheAllocFreeFn free_fn)
      : object_ptr_(object_ptr),
        reset_fn_(reset_fn),
        alloc_fn_(alloc_fn),
        free_fn_(free_fn) {}

 private:
  void reset() override { reset_fn_(object_ptr_); }

  void* alloc(const size_t size) override {
    return alloc_fn_(object_ptr_, size);
  }

  void free(void* ptr) override { free_fn_(object_ptr_, ptr); }

  void* object_ptr_;
  ForwardVtableTileCacheAllocResetFn reset_fn_;
  ForwardVtableTileCacheAllocAllocFn alloc_fn_;
  ForwardVtableTileCacheAllocFreeFn free_fn_;
};

dtTileCacheAlloc* CreateForwardedTileCacheAlloc(
    void* object_ptr, ForwardVtableTileCacheAllocResetFn reset_fn,
    ForwardVtableTileCacheAllocAllocFn alloc_fn,
    ForwardVtableTileCacheAllocFreeFn free_fn) {
  return new ForwardVtableTileCacheAlloc(object_ptr, reset_fn, alloc_fn,
                                         free_fn);
}

class ForwardVtableTileCacheCompressor : public dtTileCacheCompressor {
 public:
  ForwardVtableTileCacheCompressor(
      void* object_ptr,
      ForwardVtableTileCacheCompressorMaxCompressedSizeFn
          max_compressed_size_fn,
      ForwardVtableTileCacheCompressorCompressFn compress_fn,
      ForwardVtableTileCacheCompressorDecompressFn decompress_fn)
      : object_ptr_(object_ptr),
        max_compressed_size_fn_(max_compressed_size_fn),
        compress_fn_(compress_fn),
        decompress_fn_(decompress_fn) {}

 private:
  int maxCompressedSize(const int bufferSize) override {
    return max_compressed_size_fn_(object_ptr_, bufferSize);
  }
  dtStatus compress(const unsigned char* buffer, const int bufferSize,
                    unsigned char* compressed, const int maxCompressedSize,
                    int* compressedSize) override {
    return compress_fn_(object_ptr_, buffer, bufferSize, compressed,
                        maxCompressedSize, compressedSize);
  }
  dtStatus decompress(const unsigned char* compressed, const int compressedSize,
                      unsigned char* buffer, const int maxBufferSize,
                      int* bufferSize) override {
    return decompress_fn_(object_ptr_, compressed, compressedSize, buffer,
                          maxBufferSize, bufferSize);
  }

  void* object_ptr_;
  ForwardVtableTileCacheCompressorMaxCompressedSizeFn max_compressed_size_fn_;
  ForwardVtableTileCacheCompressorCompressFn compress_fn_;
  ForwardVtableTileCacheCompressorDecompressFn decompress_fn_;
};

dtTileCacheCompressor* CreateForwardedTileCacheCompressor(
    void* object_ptr,
    ForwardVtableTileCacheCompressorMaxCompressedSizeFn max_compressed_size_fn,
    ForwardVtableTileCacheCompressorCompressFn compress_fn,
    ForwardVtableTileCacheCompressorDecompressFn decompress_fn) {
  return new ForwardVtableTileCacheCompressor(
      object_ptr, max_compressed_size_fn, compress_fn, decompress_fn);
}

void DeleteTileCacheCompressor(dtTileCacheCompressor* compressor) {
  delete compressor;
}
