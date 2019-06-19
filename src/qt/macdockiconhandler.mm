// Copyright (c) 2011-2018 The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include "macdockiconhandler.h"

#undef slots
#include <objc/objc.h>
#include <objc/message.h>

static MacDockIconHandler *s_instance = nullptr;

bool dockClickHandler(id self, SEL _cmd, ...) {
    Q_UNUSED(self)
    Q_UNUSED(_cmd)

    Q_EMIT s_instance->dockIconClicked();

    // Return NO (false) to suppress the default macOS actions
    return false;
}

void setupDockClickHandler() {
    id app = objc_msgSend((id)objc_getClass("NSApplication"), sel_registerName("sharedApplication"));
    id delegate = objc_msgSend(app, sel_registerName("delegate"));
    Class delClass = (Class)objc_msgSend(delegate, sel_registerName("class"));
    SEL shouldHandle = sel_registerName("applicationShouldHandleReopen:hasVisibleWindows:");
    class_replaceMethod(delClass, shouldHandle, (IMP)dockClickHandler, "B@:");
}

MacDockIconHandler::MacDockIconHandler() : QObject()
{
    setupDockClickHandler();
}

MacDockIconHandler *MacDockIconHandler::instance()
{
    if (!s_instance)
        s_instance = new MacDockIconHandler();
    return s_instance;
}

void MacDockIconHandler::cleanup()
{
    delete s_instance;
}
