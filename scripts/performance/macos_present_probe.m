// Local benchmark instrumentation only. No library code or installed framework is changed.
// Build with: xcrun clang -O2 -fobjc-arc -fblocks -dynamiclib ... -framework Foundation
//             -framework Metal -framework QuartzCore -o /tmp/finstack-present-probe.dylib
// Inject only into the benchmark process with DYLD_INSERT_LIBRARIES and FINSTACK_METAL_LOG.
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>
#import <objc/runtime.h>
#include <pthread.h>
#include <stdio.h>
#include <time.h>

static FILE *output;
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static id (*originalDrawable)(id, SEL);
static id (*originalBuffer)(id, SEL);
static _Thread_local unsigned long long currentDrawable;
static _Thread_local unsigned long long currentLayer;
static unsigned long long hostNS(void) { return (unsigned long long)(CACurrentMediaTime()*1e9); }
static unsigned long long unixNS(void) {
    struct timespec t; clock_gettime(CLOCK_REALTIME,&t);
    return (unsigned long long)t.tv_sec*1000000000ULL+(unsigned long long)t.tv_nsec;
}
static id nextDrawable(id layer, SEL selector) {
    const unsigned long long begin=hostNS();
    id<CAMetalDrawable> drawable=originalDrawable(layer,selector);
    if (!drawable) return nil;
    const unsigned long long handle=(unsigned long long)(__bridge void *)layer;
    const unsigned long long identifier=drawable.drawableID;
    currentLayer=handle; currentDrawable=identifier;
    pthread_mutex_lock(&lock);
    fprintf(output,"{\"event\":\"metal-drawable\",\"layer\":\"%llx\",\"drawable\":%llu,\"begin_host_ns\":\"%llu\",\"host_ns\":\"%llu\",\"unix_ns\":\"%llu\"}\n",handle,identifier,begin,hostNS(),unixNS());
    pthread_mutex_unlock(&lock);
    [drawable addPresentedHandler:^(id<MTLDrawable> d) {
        pthread_mutex_lock(&lock);
        fprintf(output,"{\"event\":\"metal-presented\",\"layer\":\"%llx\",\"drawable\":%llu,\"presented_host_ns\":\"%llu\",\"callback_host_ns\":\"%llu\"}\n",handle,identifier,(unsigned long long)(d.presentedTime*1e9),hostNS());
        pthread_mutex_unlock(&lock);
    }];
    return drawable;
}
static id commandBuffer(id queue, SEL selector) {
    id<MTLCommandBuffer> buffer=originalBuffer(queue,selector);
    const unsigned long long layer=currentLayer,drawable=currentDrawable;
    const unsigned long long created=hostNS();
    [buffer addCompletedHandler:^(id<MTLCommandBuffer> b) {
        pthread_mutex_lock(&lock);
        fprintf(output,"{\"event\":\"metal-completed\",\"layer\":\"%llx\",\"drawable\":%llu,\"created_host_ns\":\"%llu\",\"gpu_start_ns\":\"%llu\",\"gpu_end_ns\":\"%llu\",\"status\":%lu}\n",layer,drawable,created,(unsigned long long)(b.GPUStartTime*1e9),(unsigned long long)(b.GPUEndTime*1e9),(unsigned long)b.status);
        pthread_mutex_unlock(&lock);
    }];
    return buffer;
}
__attribute__((constructor)) static void install(void) {
    const char *path=getenv("FINSTACK_METAL_LOG");
    if (!path) return;
    output=fopen(path,"w");
    if (!output) abort();
    setvbuf(output,NULL,_IOLBF,0);
    @autoreleasepool {
        Method drawable=class_getInstanceMethod([CAMetalLayer class],@selector(nextDrawable));
        originalDrawable=(void *)method_getImplementation(drawable);
        method_setImplementation(drawable,(IMP)nextDrawable);
        id<MTLDevice> device=MTLCreateSystemDefaultDevice();
        id<MTLCommandQueue> queue=[device newCommandQueue];
        Method buffer=class_getInstanceMethod(object_getClass(queue),@selector(commandBuffer));
        if (!buffer) abort();
        originalBuffer=(void *)method_getImplementation(buffer);
        method_setImplementation(buffer,(IMP)commandBuffer);
        fprintf(output,"{\"event\":\"metal-probe\",\"version\":1,\"host_ns\":\"%llu\",\"unix_ns\":\"%llu\"}\n",hostNS(),unixNS());
    }
}
