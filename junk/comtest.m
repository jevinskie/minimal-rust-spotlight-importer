#include <CoreFoundation/CFBase.h>
#import <CoreFoundation/CFPlugInCOM.h>
#include <CoreFoundation/CFString.h>
#include <CoreFoundation/CFUUID.h>
#import <CoreFoundation/CoreFoundation.h>
#import <CoreServices/CoreServices.h>
#import <Foundation/Foundation.h>

#undef NDEBUG
#include <assert.h>

#include <dlfcn.h>
#include <stdio.h>

typedef void *(*queryintf_t)(void *thisPointer, REFIID iid, LPVOID *ppv);
typedef void *(*factory_t)(CFAllocatorRef allocator, CFUUIDRef typeID);
typedef CFStringRef (*ret_cfstr_t)(void);

CF_EXPORT Boolean _CFIsObjC(CFTypeID typeID, void *obj);

// true if the obj isn't the native ObjC type
#define CF_IS_OBJC(typeID, obj) _CFIsObjC(typeID, obj)

int main(int argc, const char **argv) {
    if (argc != 2) {
        return 1;
    }
    @autoreleasepool {
        unsigned char hb[] = {'h', 'e', 'l', 'l', 'o'};
        CFStringRef hcfstr = CFStringCreateWithBytes(kCFAllocatorDefault, hb, sizeof(hb),
                                                     kCFStringEncodingUTF8, false);
        // bool isobjc = CF_IS_OBJC(CFStringGetTypeID(), hcfstr);
        // printf("hcfstr isobjc: %d\n", isobjc);
        printf("hcfstr: %p\n", hcfstr);
        NSLog(@"NSLOG hcfstr: %@", hcfstr);
        CFShow(hcfstr);
        const char *hcstr = CFStringGetCStringPtr(hcfstr, kCFStringEncodingUTF8);
        printf("hcstr: '%s'\n", hcstr);
        void *h = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL | RTLD_FIRST);
        assert(h);
        factory_t ff = (factory_t)dlsym(h, "MetadataImporterPluginFactory");
        assert(ff);
        printf("ff: %p\n", ff);
        void *rff = ff(kCFAllocatorDefault, kMDImporterTypeID);
        printf("rff: %p\n", rff);
        ret_cfstr_t rcfs = (ret_cfstr_t)dlsym(h, "ReturnCFString");
        assert(rcfs);
        CFStringRef rs = rcfs();
        printf("rs: %p\n", rs);
        CFShow(rs);
        printf("CFGetRetainCount(rs): %lu\n", CFGetRetainCount(rs));
        NSLog(@"NSLOG rs: %@", rs);
        // isobjc = CF_IS_OBJC(CFStringGetTypeID(), rs);
        // printf("rs isobjc: %d\n", isobjc);
        CFStringRef cfdesc      = CFCopyDescription(rs);
        const char *cfdesc_cstr = CFStringGetCStringPtr(cfdesc, kCFStringEncodingUTF8);
        printf("cfdesc: '%s'\n", cfdesc_cstr);
        const char *rsc = CFStringGetCStringPtr(rs, kCFStringEncodingUTF8);
        printf("rsc: '%s'\n", rsc);
        void *p          = NULL;
        CFUUIDBytes unkb = CFUUIDGetUUIDBytes(IUnknownUUID);
        void *r          = ff(kCFAllocatorDefault, IUnknownUUID);
        printf("r: %p p: %p\n", r, p);
        void **pr = (void **)r;
        for (int i = 0; i < 4; ++i) {
            printf("pr[%d] = %p\n", i, pr[i]);
        }
    }
    return 0;
}
