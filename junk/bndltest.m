#include <CoreFoundation/CFPlugInCOM.h>
#include <CoreFoundation/CoreFoundation.h>
#import <CoreServices/CoreServices.h>
#import <Foundation/Foundation.h>

#include <dlfcn.h>
#include <stdio.h>

#undef NDEBUG
#include <assert.h>

/*!
        @constant kMDImporterTypeID The importer only loads CFPlugIns of type
        kMDImporterTypeID - 8B08C4BF-415B-11D8-B3F9-0003936726FC

        @constant kMDImporterInterfaceID Importers must implement this
        Interface - 6EBC27C4-899C-11D8-84A3-0003936726FC

        @constant kMDExporterInterfaceID Exporters can optionaly also implement this
        Interface - B41C6074-7DFB-4057-969D-31C8E861A8D4

        @constant kMDImporterURLInterfaceID Importers can optionaly also implement this
        Interface - B41C6074-7DFB-4057-969D-31C8E861A8D4

        @constant kMDImporterBundleWrapperURLInterfaceID Importers can optionaly also implement this
        Interface - CF76374B-0C83-47C5-AB2F-7B950884670A

*/

#define kMinimalImporterUUID                                                                      \
    CFUUIDGetConstantUUIDWithBytes(kCFAllocatorDefault, 0xd8, 0x78, 0x57, 0xf7, 0xb0, 0xc0, 0x4c, \
                                   0x70, 0x9b, 0x8f, 0x2e, 0x3d, 0x8e, 0x55, 0x19, 0x8c)

#define kDsymImporterUUID                                                                         \
    CFUUIDGetConstantUUIDWithBytes(kCFAllocatorDefault, 0xfc, 0x09, 0x57, 0xc6, 0x10, 0xa1, 0x46, \
                                   0x9b, 0xaa, 0x54, 0x5a, 0x9f, 0xfe, 0x71, 0x8a, 0x93)

typedef void *(*queryintf_t)(void *thisPointer, REFIID iid, LPVOID *ppv);
typedef void *(*factory_t)(CFAllocatorRef allocator, CFUUIDRef typeID);
typedef CFStringRef (*ret_cfstr_t)(void);

typedef struct MetadataImporterPlugin_s {
    MDImporterInterfaceStruct *conduitInterface;
    CFUUIDRef factoryID;
    UInt32 refCount;
} MetadataImporterPlugin_t;

int main(int argc, const char **argv) {
    if (argc != 3) {
        return 1;
    }
    @autoreleasepool {
        NSURL *bndl_url = [NSURL fileURLWithPath:@(argv[1]) isDirectory:YES];
        assert(bndl_url);
        printf("bndl_url: %s\n", bndl_url.absoluteString.UTF8String);

        NSString *file_to_import = @(argv[2]);
        assert(file_to_import);
        printf("file_to_import: %s\n", file_to_import.UTF8String);

        CFPlugInRef plugin = CFPlugInCreate(kCFAllocatorDefault, (__bridge CFURLRef)bndl_url);
        assert(plugin);
        CFShow(plugin);

        NSArray *factories = CFBridgingRelease(
            CFPlugInFindFactoriesForPlugInTypeInPlugIn(kMDImporterTypeID, plugin));
        NSLog(@"factories: %@", factories);
        assert(factories.count == 1);

        IUnknownVTbl **iunknown;
        //  Use the factory ID to get an IUnknown interface. Here the plug-in code is loaded.
        iunknown = (IUnknownVTbl **)CFPlugInInstanceCreate(
            kCFAllocatorDefault, (__bridge CFUUIDRef)factories[0], kMDImporterTypeID);
        printf("&iunknown: %p\n", iunknown);
        assert(iunknown);
        printf("iunknown: %p\n", *iunknown);
        printf("iunknown->_reserved: %p &p: %p\n", (*iunknown)->_reserved,
               &((*iunknown)->_reserved));
        printf("iunknown->QueryInterface: %p &p: %p\n", (*iunknown)->QueryInterface,
               &((*iunknown)->QueryInterface));
        printf("iunknown->AddRef: %p &p: %p\n", (*iunknown)->AddRef, &((*iunknown)->AddRef));
        printf("iunknown->Release: %p &p: %p\n", (*iunknown)->Release, &((*iunknown)->Release));
        assert(iunknown);
        MDImporterInterfaceStruct **mdip = NULL;
        HRESULT hres;
        assert((*iunknown)->QueryInterface);
        hres = (*iunknown)->QueryInterface(iunknown, CFUUIDGetUUIDBytes(kMDImporterInterfaceID),
                                           (LPVOID *)(&mdip));
        printf("hres: %d %x mdip: %p\n", hres, (uint32_t)hres, mdip);
        fflush(stdout);
        if (mdip) {
            printf("*mdip: %p\n", *mdip);
        }
        printf("before (*iunknown)->Release(iunknown);\n");
        fflush(stdout);
        assert((*iunknown)->Release);
        (*iunknown)->Release(iunknown);
        printf("&iunknown: %p\n", iunknown);
        printf("iunknown: %p\n", *iunknown);
        printf("iunknown->_reserved: %p &p: %p\n", (*iunknown)->_reserved,
               &((*iunknown)->_reserved));
        printf("iunknown->QueryInterface: %p &p: %p\n", (*iunknown)->QueryInterface,
               &((*iunknown)->QueryInterface));
        printf("iunknown->AddRef: %p &p: %p\n", (*iunknown)->AddRef, &((*iunknown)->AddRef));
        printf("iunknown->Release: %p &p: %p\n", (*iunknown)->Release, &((*iunknown)->Release));
        printf("after (*iunknown)->Release(iunknown);\n");
        fflush(stdout);
        assert(mdip);
        MDImporterInterfaceStruct *mdi = ((MetadataImporterPlugin_t *)mdip)->conduitInterface;
        assert(mdi);
        printf("mdi AddRef: %p &p: %p\n", mdi->AddRef, &(mdi->AddRef));
        printf("mdi Release: %p &p: %p\n", mdi->Release, &(mdi->Release));
        printf("mdi ImporterImportData: %p &p: %p\n", mdi->ImporterImportData,
               &(mdi->ImporterImportData));
        NSMutableDictionary *dict = NSMutableDictionary.new;
        NSLog(@"dict before: %@", dict);
        // NSString *uti = @"com.apple.xcode.dsym";
        NSString *uti = @"vin.je.rich";
        fflush(stdout);
        assert(mdi->ImporterImportData);
        Boolean import_res = mdi->ImporterImportData(mdi, (__bridge CFMutableDictionaryRef)dict,
                                                     (__bridge CFStringRef)uti,
                                                     (__bridge CFStringRef)file_to_import);
        printf("import_res: %d\n", import_res);
        NSLog(@"dict after: %@", dict);
        assert(mdi->Release);
        mdi->Release(mdi);
        printf("mdi final release done\n");
        CFRelease(plugin);
        printf("released plugin\n");
    }
    return 0;
}
