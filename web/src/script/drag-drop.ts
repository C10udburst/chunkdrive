function showOverlay() {
    const overlay = document.getElementById('drag-overlay');
    overlay.classList.add('show');
}

function hideOverlay() {
    const overlay = document.getElementById('drag-overlay');
    overlay.classList.remove('show');
}

function dragLeave(e: DragEvent) {
    if (e.pageX === 0 && e.pageY === 0) {
        hideOverlay();
    }
}

function drop(e: DragEvent) {
    hideOverlay();

    const form = document.querySelector('form.file-upload') as HTMLFormElement;
    const input = form.querySelector('input[type="file"]') as HTMLInputElement;

    let files = e.dataTransfer.files;

    if (files.length === 0) {
        return;
    }

    if (files.length > 1) {
        alert('Please only upload one file at a time.');
        return;
    }
    
    input.files = files;
    form.submit();
}

function preventDefaults (e: Event) {
    e.preventDefault()
    e.stopPropagation()
  }

export function initDragDrop() {
    let overlay = document.createElement('div');
    overlay.classList.add('overlay');
    overlay.id = 'drag-overlay';

    let body = document.querySelector('body');
    body.appendChild(overlay);

    window.addEventListener('dragenter', showOverlay, false);
    window.addEventListener('dragover', showOverlay, false);
    window.addEventListener('drop', drop, false);
    window.addEventListener('dragleave', dragLeave, false);

    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        window.addEventListener(eventName, preventDefaults, false);
    });
}