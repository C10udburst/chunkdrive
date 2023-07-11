export function initDestructiveWarn() {
    document.querySelectorAll('.destructive form').forEach(form => {
        form.addEventListener('submit', event => {
            let inode = form.getAttribute('action').split('/').pop();
            let name = inode.split('$').pop();

            let message = `You are about to irreversibly change "${name}". Are you sure?`;
            if (!confirm(message)) {
                event.preventDefault();
            }
        });
    });
}