#import <Foundation/Foundation.h>
#import <GameController/GameController.h>
#import <CoreFoundation/CoreFoundation.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    uint64_t identifier;
    char name[128];
    float axes[4];
    float buttons[17];
} IW4LController;

// Read-only polling on the main thread. No OS input synthesis, driver
// installation, Steam settings, exclusive HID access or permission changes.
uint32_t iw4l_poll_controllers(IW4LController *out, uint32_t capacity, int pump) {
    @autoreleasepool {
        [GCController setShouldMonitorBackgroundEvents:YES];
        // The headless checker has no AppKit runner. Windowed games already
        // pump their main run loop through winit.
        if(pump) CFRunLoopRunInMode(kCFRunLoopDefaultMode,0.001,true);
        uint32_t count=0;
        for(GCController *controller in GCController.controllers) {
            GCExtendedGamepad *pad=controller.extendedGamepad;
            if(!pad || count>=capacity) continue;
            IW4LController *s=&out[count++];
            memset(s,0,sizeof(*s));
            s->identifier=(uint64_t)(uintptr_t)(__bridge void *)controller;
            NSString *name=controller.vendorName ?: @"Apple GameController";
            strncpy(s->name,name.UTF8String,sizeof(s->name)-1);
            s->axes[0]=pad.leftThumbstick.xAxis.value;
            s->axes[1]=pad.leftThumbstick.yAxis.value;
            s->axes[2]=pad.rightThumbstick.xAxis.value;
            s->axes[3]=pad.rightThumbstick.yAxis.value;
            // Physical positions, consistently mapped across PS/Xbox/Switch.
            s->buttons[0]=pad.buttonA.value;
            s->buttons[1]=pad.buttonB.value;
            s->buttons[2]=pad.buttonY.value;
            s->buttons[3]=pad.buttonX.value;
            s->buttons[4]=pad.leftShoulder.value;
            s->buttons[5]=pad.rightShoulder.value;
            s->buttons[6]=pad.leftTrigger.value;
            s->buttons[7]=pad.rightTrigger.value;
            s->buttons[8]=pad.buttonOptions.value;
            s->buttons[9]=pad.buttonMenu.value;
            s->buttons[10]=pad.buttonHome.value;
            s->buttons[11]=pad.leftThumbstickButton.value;
            s->buttons[12]=pad.rightThumbstickButton.value;
            s->buttons[13]=pad.dpad.up.value;
            s->buttons[14]=pad.dpad.down.value;
            s->buttons[15]=pad.dpad.left.value;
            s->buttons[16]=pad.dpad.right.value;
        }
        return count;
    }
}
