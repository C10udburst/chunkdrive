function beforeUnload(e: BeforeUnloadEvent) {
    if (window['uploading']) {
        const msg = `You are uploading "${window['uploading']}". Are you sure you want to leave?`;
        (e || window.event).returnValue = msg;
        return msg;
    }
}

export function upload(form: HTMLFormElement) {
    const input = form.querySelector('input[type=file]') as HTMLInputElement;
    const file = input.files[0];
    if (file == null)
        return;

    window['uploading'] = file.name;

    const overlay = document.getElementById('progress-overlay') as HTMLDivElement;
    const progress = overlay.querySelector('.progress-bar') as HTMLDivElement;
    const filename = overlay.querySelector('.filename') as HTMLSpanElement;

    filename.textContent = file.name;
    overlay.classList.add('show');

    const xhr = new XMLHttpRequest();
    xhr.open('POST', form.action, true);
    xhr.upload.onprogress = (e: ProgressEvent) => {
        if (e.lengthComputable) {
            const percent = (e.loaded / e.total) * 100;
            progress.style.width = percent + '%';
        }
    }
    xhr.onload = () => {
        delete window['uploading'];
        window.location.href = xhr.responseURL;
    }
    xhr.send(new FormData(form));
}

function onSubmit(e: SubmitEvent) {
    e.preventDefault();

    upload(e.target as HTMLFormElement);
}

function initOverlay() {
    const overlay = document.createElement('div');
    overlay.classList.add('overlay');
    overlay.id = 'progress-overlay';

    const filename = document.createElement('span');
    filename.classList.add('filename');
    overlay.appendChild(filename);

    const progressContainer = document.createElement('div');
    progressContainer.classList.add('progress-container');
    const progress = document.createElement('div');
    progress.classList.add('progress-bar');
    progressContainer.appendChild(progress);
    overlay.appendChild(progressContainer);


    const body = document.querySelector('body');
    body.appendChild(overlay);
}

export function initFormUpload() {
    const form = document.querySelector('form.file-upload');
    if (form == null)
        return;
    initOverlay();
    form.addEventListener('submit', onSubmit, false);
    window.addEventListener('beforeunload', beforeUnload, false);
}