#import <Cocoa/Cocoa.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <signal.h>
#include <unistd.h>

static NSString *Version = @"0.9.0";
static NSString *RuntimeVersion = @"0.8.0";
static NSURL *RuntimeRoot(void) {
    NSString *custom=NSProcessInfo.processInfo.environment[@"IW4L_INVASION_HOME"];
    if(custom.length) return [NSURL fileURLWithPath:custom isDirectory:YES];
    NSURL *support=[NSFileManager.defaultManager URLsForDirectory:NSApplicationSupportDirectory inDomains:NSUserDomainMask].firstObject;
    return [[support URLByAppendingPathComponent:@"Rust Minecraft Invasion" isDirectory:YES] URLByAppendingPathComponent:RuntimeVersion isDirectory:YES];
}
static NSURL *GameBinary(void) {return [NSBundle.mainBundle.resourceURL URLByAppendingPathComponent:@"iw4l"];}
static BOOL Exists(NSString *path) {return [NSFileManager.defaultManager fileExistsAtPath:path];}
static BOOL ValidMW2(NSString *path) {
    return path.length && Exists([path stringByAppendingPathComponent:@"zone/english/mp_rust.ff"])
        && Exists([path stringByAppendingPathComponent:@"zone/english/common_mp.ff"])
        && Exists([path stringByAppendingPathComponent:@"main"]);
}
static NSString *RunReadOnly(NSString *exe, NSArray *args, int *status) {
    NSTask *task=[NSTask new];task.executableURL=[NSURL fileURLWithPath:exe];task.arguments=args;task.currentDirectoryURL=RuntimeRoot();
    NSPipe *pipe=NSPipe.pipe;task.standardOutput=pipe;task.standardError=pipe;
    NSError *error=nil;if(![task launchAndReturnError:&error]){if(status)*status=-1;return error.localizedDescription;}
    NSData *data=[pipe.fileHandleForReading readDataToEndOfFile];[task waitUntilExit];if(status)*status=task.terminationStatus;
    return [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding] ?: @"";
}
@interface InvasionApp : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property NSWindow *window;
@property NSTextField *status;
@property NSTextField *folder;
@property NSButton *choose;
@property NSButton *prepare;
@property NSButton *play;
@property NSButton *check;
@property NSTask *task;
@property NSString *games;
@property BOOL quitting;
@property BOOL preparing;
@property int perfFD;
@property int captureFD;
@property int instanceFD;
@end
@implementation InvasionApp
- (NSTextField *)label:(NSString *)text rect:(NSRect)rect size:(CGFloat)size {
    NSTextField *label=[NSTextField wrappingLabelWithString:text];label.frame=rect;label.font=[NSFont systemFontOfSize:size];return label;
}
- (NSURL *)configURL {return [RuntimeRoot() URLByAppendingPathComponent:@"settings.plist"];}
- (BOOL)resourcesReady {return Exists([[RuntimeRoot() URLByAppendingPathComponent:@"iw4l-artifacts/minecraft-26.3/.complete"] path]);}
- (void)applicationDidFinishLaunching:(NSNotification *)note {
    (void)note;self.perfFD=-1;self.captureFD=-1;self.instanceFD=-1;
    [NSFileManager.defaultManager createDirectoryAtURL:RuntimeRoot() withIntermediateDirectories:YES attributes:nil error:nil];
    NSDictionary *config=[NSDictionary dictionaryWithContentsOfURL:self.configURL];self.games=config[@"MW2Folder"] ?: @"";
    self.window=[[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,650,390) styleMask:NSWindowStyleMaskTitled|NSWindowStyleMaskClosable|NSWindowStyleMaskMiniaturizable backing:NSBackingStoreBuffered defer:NO];
    self.window.title=@"Rust Minecraft Invasion — Survival";self.window.delegate=self;self.window.releasedWhenClosed=NO;[self.window center];
    NSView *view=self.window.contentView;
    NSTextField *title=[self label:@"Rust Minecraft Invasion" rect:NSMakeRect(25,326,600,38) size:27];title.font=[NSFont boldSystemFontOfSize:27];[view addSubview:title];
    [view addSubview:[self label:@"134 mobs • All 15 killstreaks • Intervention Quickscope" rect:NSMakeRect(25,292,600,28) size:15]];
    [view addSubview:[self label:@"Select your owned Windows MW2 (2009) game folder. Prepare downloads Minecraft resources from Mojang. Open Game Menu when you’re ready; you start the match yourself." rect:NSMakeRect(25,229,600,58) size:14]];
    self.folder=[self label:self.games.length ? self.games : @"No MW2 folder selected" rect:NSMakeRect(25,183,600,39) size:12];[view addSubview:self.folder];
    self.status=[self label:@"" rect:NSMakeRect(25,89,600,84) size:14];[view addSubview:self.status];
    self.choose=[NSButton buttonWithTitle:@"Choose MW2 Folder" target:self action:@selector(chooseFolder:)];self.choose.frame=NSMakeRect(20,25,170,37);[view addSubview:self.choose];
    self.prepare=[NSButton buttonWithTitle:@"Prepare Files" target:self action:@selector(prepareFiles:)];self.prepare.frame=NSMakeRect(192,25,125,37);[view addSubview:self.prepare];
    self.check=[NSButton buttonWithTitle:@"Check Setup" target:self action:@selector(checkSetup:)];self.check.frame=NSMakeRect(319,25,125,37);[view addSubview:self.check];
    self.play=[NSButton buttonWithTitle:@"Open Game Menu" target:self action:@selector(playGame:)];self.play.frame=NSMakeRect(446,25,184,37);[view addSubview:self.play];
    [self refresh];[self.window makeKeyAndOrderFront:nil];[NSApp activate];
}
- (void)refresh {
    BOOL ready=ValidMW2(self.games)&&self.resourcesReady;
    self.play.enabled=ready&&!self.task.running;self.choose.enabled=!self.task.running;self.prepare.enabled=!self.task.running;self.check.enabled=!self.task.running;
    if(!self.task.running) self.status.stringValue=ready ? @"Ready. The game is closed. In your match, choose Intervention Quickscope. D-pad Right / 4 deploys the next earned killstreak." : @"Setup needed: choose your MW2 folder and click Prepare Files. No game starts during preparation.";
}
- (void)chooseFolder:(id)sender {
    (void)sender;NSOpenPanel *panel=NSOpenPanel.openPanel;panel.canChooseFiles=NO;panel.canChooseDirectories=YES;panel.allowsMultipleSelection=NO;panel.message=@"Choose the Windows MW2 (2009) folder containing main and zone/english/mp_rust.ff.";
    [panel beginSheetModalForWindow:self.window completionHandler:^(NSModalResponse result){
        if(result!=NSModalResponseOK)return;
        NSString *path=panel.URL.path;
        if(!ValidMW2(path)){self.status.stringValue=@"That folder is missing the English MW2 multiplayer data. Select the game’s installation folder, with main and zone/english inside.";return;}
        self.games=path;NSMutableDictionary *config=[[NSDictionary dictionaryWithContentsOfURL:self.configURL] mutableCopy] ?: [NSMutableDictionary new];config[@"MW2Folder"]=path;[config writeToURL:self.configURL atomically:YES];self.folder.stringValue=path;[self refresh];
    }];
}
- (void)releaseLocks {
    if(self.captureFD>=0){flock(self.captureFD,LOCK_UN);close(self.captureFD);self.captureFD=-1;}
    if(self.perfFD>=0){flock(self.perfFD,LOCK_UN);close(self.perfFD);self.perfFD=-1;}
    if(self.instanceFD>=0){flock(self.instanceFD,LOCK_UN);close(self.instanceFD);self.instanceFD=-1;}
}
- (NSString *)claimRenderer {
    struct stat console;if(stat("/dev/console",&console)!=0 || console.st_uid==0)return @"Log into the Mac desktop before opening the game.";
    NSDictionary *env=NSProcessInfo.processInfo.environment;
    NSDictionary *config=[NSDictionary dictionaryWithContentsOfURL:self.configURL];
    id configuredValue=config[@"RendererSlotDirectory"];
    if(configuredValue && (![configuredValue isKindOfClass:NSString.class] || ![configuredValue length]))return @"The configured renderer directory is invalid.";
    NSString *configured=configuredValue;
    NSString *explicitRoot=[env[@"GPU_SLOT_DIR"] length] ? env[@"GPU_SLOT_DIR"] : ([env[@"GPU_LOCK_DIR"] length] ? env[@"GPU_LOCK_DIR"] : nil);
    NSString *(^canonical)(NSString *)=^NSString *(NSString *path){return [[path stringByExpandingTildeInPath] stringByResolvingSymlinksInPath];};
    if(configured.length && explicitRoot.length && ![canonical(configured) isEqualToString:canonical(explicitRoot)])return @"Renderer directory settings disagree. Resolve the configuration before launching.";
    NSString *base=canonical(configured.length ? configured : (explicitRoot ?: [NSHomeDirectory() stringByAppendingPathComponent:@".cache/gpu-slot"]));
    if(!base.isAbsolutePath)return @"The renderer directory must be an absolute path.";
    if((configured.length || explicitRoot.length) && !Exists([base stringByAppendingPathComponent:@"locks/perf.lock"]))return @"The configured shared renderer protocol is missing. No fallback directory was created.";
    if(Exists([base stringByAppendingPathComponent:@"PAUSED"])||Exists([NSHomeDirectory() stringByAppendingPathComponent:@"ralph-slots/PAUSED"]))return @"Renderer launches are paused on this Mac. Leave that pause in place.";
    NSString *locks=[base stringByAppendingPathComponent:@"locks"];
    [NSFileManager.defaultManager createDirectoryAtPath:locks withIntermediateDirectories:YES attributes:nil error:nil];
    self.perfFD=open([[locks stringByAppendingPathComponent:@"perf.lock"] fileSystemRepresentation],O_CREAT|O_RDWR,0600);
    if(self.perfFD<0 || flock(self.perfFD,LOCK_SH|LOCK_NB)!=0){[self releaseLocks];return @"A graphics performance task is using the shared renderer lock. Try again later.";}
    for(int i=0;i<2;i++){
        int fd=open([[locks stringByAppendingPathComponent:[NSString stringWithFormat:@"capture.%d.lock",i]] fileSystemRepresentation],O_CREAT|O_RDWR,0600);
        if(fd>=0 && flock(fd,LOCK_EX|LOCK_NB)==0){self.captureFD=fd;break;}if(fd>=0)close(fd);
    }
    if(self.captureFD<0){[self releaseLocks];return @"Both shared renderer slots are busy. Try again after another game/editor closes.";}
    int code;NSString *output=RunReadOnly(@"/bin/ps",@[@"-axo",@"stat=,comm="],&code);int count=0;
    NSSet *engines=[NSSet setWithArray:@[@"iw4l",@"unrealeditor",@"unrealgame",@"unity",@"godot",@"blender",@"robloxstudio",@"robloxplayer",@"gta5.exe",@"gta5_enhanced.exe",@"eldenring.exe",@"darksoulsremastered.exe",@"iw4mp.exe"]];
    for(NSString *row in [output componentsSeparatedByString:@"\n"]){
        NSString *trim=[row stringByTrimmingCharactersInSet:NSCharacterSet.whitespaceCharacterSet];NSRange space=[trim rangeOfCharacterFromSet:NSCharacterSet.whitespaceCharacterSet];if(space.location==NSNotFound)continue;
        NSString *state=[trim substringToIndex:space.location];NSString *path=[[trim substringFromIndex:space.location] stringByTrimmingCharactersInSet:NSCharacterSet.whitespaceCharacterSet];
        NSString *name=path.lastPathComponent.lowercaseString;
        if([name isEqualToString:@"gta5.exe"]||[name isEqualToString:@"gta5_enhanced.exe"]){[self releaseLocks];return @"GTA requires exclusive GPU use on this setup. Close it yourself before opening Rust.";}
        if([engines containsObject:name]){count++;if([state containsString:@"E"]||[state containsString:@"Z"]){[self releaseLocks];return @"A game/editor is still exiting. Wait for it to finish.";}}
    }
    if(code!=0||count>=2){[self releaseLocks];return @"Two renderer-bearing games/editors are already open. Close one yourself before starting another.";}
    return nil;
}
- (void)run:(BOOL)prepare {
    if(self.task.running)return;
    if(!prepare){NSString *error=[self claimRenderer];if(error){self.status.stringValue=error;return;}}
    self.instanceFD=open([[RuntimeRoot() URLByAppendingPathComponent:@".instance.lock"].path fileSystemRepresentation],O_CREAT|O_RDWR,0600);
    if(self.instanceFD<0||flock(self.instanceFD,LOCK_EX|LOCK_NB)!=0){[self releaseLocks];self.status.stringValue=@"This version is already open or preparing files.";return;}
    self.preparing=prepare;self.task=[NSTask new];self.task.executableURL=GameBinary();self.task.arguments=prepare?@[@"prepare-invasion"]:@[@"menu"];self.task.currentDirectoryURL=RuntimeRoot();
    NSMutableDictionary *env=[NSProcessInfo.processInfo.environment mutableCopy];
    env[@"IW4L_GAMES"]=self.games ?: @"";env[@"IW4L_RUST_INVASION"]=@"1";env[@"IW4L_SKATE"]=@"off";env[@"IW4L_UPDATE"]=@"0";
    env[@"IW4L_SETTINGS_PATH"]=[RuntimeRoot() URLByAppendingPathComponent:@"settings.cfg"].path;
    env[@"XDG_CONFIG_HOME"]=[RuntimeRoot() URLByAppendingPathComponent:@"config"].path;env[@"XDG_CACHE_HOME"]=[RuntimeRoot() URLByAppendingPathComponent:@"cache"].path;
    for(NSString *key in @[@"IW4L_CMDS",@"IW4L_CONSOLE_CMDS",@"MINECRAFTOSS_ROOT",@"IW4L_PERF",@"IW4L_BENCH"])[env removeObjectForKey:key];
    self.task.environment=env;
    NSURL *logs=[RuntimeRoot() URLByAppendingPathComponent:@"logs" isDirectory:YES];[NSFileManager.defaultManager createDirectoryAtURL:logs withIntermediateDirectories:YES attributes:nil error:nil];
    NSURL *log=[logs URLByAppendingPathComponent:prepare?@"prepare.log":@"latest-launch.log"];
    [NSFileManager.defaultManager createFileAtPath:log.path contents:nil attributes:nil];NSFileHandle *file=[NSFileHandle fileHandleForWritingAtPath:log.path];self.task.standardOutput=file;self.task.standardError=file;
    __weak InvasionApp *weak=self;
    self.task.terminationHandler=^(NSTask *task){dispatch_async(dispatch_get_main_queue(),^{
        InvasionApp *strong=weak;if(!strong)return;[file closeFile];[strong releaseLocks];[strong refresh];
        if(strong.quitting){[NSApp replyToApplicationShouldTerminate:YES];return;}
        [strong.window makeKeyAndOrderFront:nil];
        if(task.terminationStatus!=0)strong.status.stringValue=[NSString stringWithFormat:@"%@ exited with code %d. See %@. It was not relaunched.",prepare?@"Preparation":@"Game",task.terminationStatus,log.path];
    });};
    NSError *error=nil;if(![self.task launchAndReturnError:&error]){[file closeFile];[self releaseLocks];[self refresh];self.status.stringValue=error.localizedDescription;return;}
    [self refresh];self.status.stringValue=prepare?@"Preparing resources from Mojang. This can take several minutes; progress is in logs/prepare.log. The game stays closed.":@"Game process started. You choose the map and start the match. Closing this launcher also closes its game.";
    if(!prepare)[self.window orderBack:nil];
}
- (void)prepareFiles:(id)sender {(void)sender;if(self.resourcesReady){[self refresh];return;}[self run:YES];}
- (void)playGame:(id)sender {(void)sender;if(ValidMW2(self.games)&&self.resourcesReady)[self run:NO];else [self refresh];}
- (void)checkSetup:(id)sender {
    (void)sender;int code;NSString *output=RunReadOnly(GameBinary().path,@[@"--help"],&code);
    [self refresh];if(code!=2||![output containsString:@"usage: iw4l"]){self.status.stringValue=@"macOS could not load the game executable. Download/extract the complete app again.";return;}
    if(ValidMW2(self.games)&&self.resourcesReady)self.status.stringValue=@"Setup checks passed: executable loads, MW2 data is present and Minecraft files are ready. The game is closed.";
}
- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)app {
    (void)app;if(!self.task.running){[self releaseLocks];return NSTerminateNow;}
    self.quitting=YES;[self.task terminate];self.status.stringValue=@"Closing this game safely…";
    NSTask *owned=self.task;dispatch_after(dispatch_time(DISPATCH_TIME_NOW,60*NSEC_PER_SEC),dispatch_get_main_queue(),^{if(owned.running)kill(owned.processIdentifier,SIGKILL);});
    return NSTerminateLater;
}
- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)app {(void)app;return NO;}
- (BOOL)windowShouldClose:(NSWindow *)window {(void)window;[NSApp terminate:nil];return NO;}
- (BOOL)applicationShouldHandleReopen:(NSApplication *)app hasVisibleWindows:(BOOL)visible {(void)app;if(!visible)[self.window makeKeyAndOrderFront:nil];return YES;}
@end
int main(int argc,const char *argv[]) {
    @autoreleasepool {
        if(argc>1&&strcmp(argv[1],"--bundle-check")==0){
            BOOL ok=Exists(GameBinary().path);printf("%s: portable invasion bundle %s; game is closed.\n",ok?"PASS":"FAIL",Version.UTF8String);return ok?0:1;
        }
        NSApplication *app=NSApplication.sharedApplication;app.activationPolicy=NSApplicationActivationPolicyRegular;
        [NSProcessInfo.processInfo disableSuddenTermination];[NSProcessInfo.processInfo disableAutomaticTermination:@"Own the game child process"];
        InvasionApp *delegate=[InvasionApp new];app.delegate=delegate;
        NSMenu *bar=[NSMenu new];NSMenuItem *item=[NSMenuItem new];NSMenu *menu=[NSMenu new];[menu addItemWithTitle:@"Quit Rust Minecraft Invasion" action:@selector(terminate:) keyEquivalent:@"q"];item.submenu=menu;[bar addItem:item];app.mainMenu=bar;
        [app run];return 0;
    }
}
