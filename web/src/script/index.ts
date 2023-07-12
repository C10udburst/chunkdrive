import { initDragDrop } from "./drag-drop";
import { initDestructiveWarn } from "./delete-warn";
import { initRememberTheme } from "./remember-theme";
import { initFormUpload } from "./form-upload";

window.addEventListener('load', () => {
    initRememberTheme();
    if (!window.config.readonly) {
        initDragDrop();
        initDestructiveWarn();
        initFormUpload();
    }
});