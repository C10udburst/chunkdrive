export function initRememberTheme() {
    const themeSwitch = document.getElementById('theme-switcher') as HTMLInputElement;
    const invertTheme = localStorage.getItem('invertTheme');

    if (invertTheme === 'true') {
        themeSwitch.checked = true;
    }

    themeSwitch.addEventListener('change', () => {
        if (themeSwitch.checked) {
            localStorage.setItem('invertTheme', 'true');
        } else {
            localStorage.setItem('invertTheme', 'false');
        }
    });
}