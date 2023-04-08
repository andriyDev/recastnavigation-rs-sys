#pragma once

// Recast definitions.

#include "Recast.h"

rcContext* CreateContext(bool state = true);

void DeleteContext(rcContext* context);

// DetourTileCache definitions.

#include "DetourNavMeshBuilder.h"
#include "DetourTileCache.h"
#include "DetourTileCacheBuilder.h"

using ForwardVtableTileCacheMeshProcessProcessFn =
    void (*)(void* object_ptr, dtNavMeshCreateParams* params,
             unsigned char* polyAreas, unsigned short* polyFlags);

dtTileCacheMeshProcess* CreateForwardedTileCacheMeshProcess(
    void* object_ptr, ForwardVtableTileCacheMeshProcessProcessFn process_fn);

void DeleteTileCacheMeshProcess(dtTileCacheMeshProcess* mesh_process);

dtTileCacheAlloc* CreateDefaultTileCacheAlloc();

void DeleteTileCacheAlloc(dtTileCacheAlloc* alloc);

using ForwardVtableTileCacheAllocResetFn = void (*)(void* object_ptr);
using ForwardVtableTileCacheAllocAllocFn = void* (*)(void* object_ptr,
                                                     const size_t size);
using ForwardVtableTileCacheAllocFreeFn = void (*)(void* object_ptr,
                                                   void* size);

dtTileCacheAlloc* CreateForwardedTileCacheAlloc(
    void* object_ptr, ForwardVtableTileCacheAllocResetFn reset_fn,
    ForwardVtableTileCacheAllocAllocFn alloc_fn,
    ForwardVtableTileCacheAllocFreeFn free_fn);

using ForwardVtableTileCacheCompressorMaxCompressedSizeFn =
    int (*)(void* object_ptr, const int bufferSize);
using ForwardVtableTileCacheCompressorCompressFn =
    dtStatus (*)(void* object_ptr, const unsigned char* buffer,
                 const int bufferSize, unsigned char* compressed,
                 const int maxCompressedSize, int* compressedSize);
using ForwardVtableTileCacheCompressorDecompressFn = dtStatus (*)(
    void* object_ptr, const unsigned char* compressed, const int compressedSize,
    unsigned char* buffer, const int maxBufferSize, int* bufferSize);

dtTileCacheCompressor* CreateForwardedTileCacheCompressor(
    void* object_ptr,
    ForwardVtableTileCacheCompressorMaxCompressedSizeFn max_compressed_size_fn,
    ForwardVtableTileCacheCompressorCompressFn compress_fn,
    ForwardVtableTileCacheCompressorDecompressFn decompress_fn);

void DeleteTileCacheCompressor(dtTileCacheCompressor* compressor);
