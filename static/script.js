document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('search-input');
    const searchButton = document.getElementById('search-button');
    const languageSelect = document.getElementById('language-select');
    const engineSelect = document.getElementById('engine-select');
    const userStatusDiv = document.getElementById('user-status');
    const themeToggle = document.getElementById('theme-toggle');

    // --- Theme Management ---
    const applyTheme = (theme) => {
        if (theme === 'dark') {
            document.body.classList.add('dark-mode');
            themeToggle.textContent = 'Light Mode';
        } else {
            document.body.classList.remove('dark-mode');
            themeToggle.textContent = 'Dark Mode';
        }
    };

    const toggleTheme = () => {
        const currentTheme = localStorage.getItem('theme');
        const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
        localStorage.setItem('theme', newTheme);
        applyTheme(newTheme);
    };

    themeToggle.addEventListener('click', toggleTheme);

    // --- Initial Load ---
    const savedTheme = localStorage.getItem('theme');
    const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;

    if (savedTheme) {
        applyTheme(savedTheme);
    } else if (prefersDark) {
        applyTheme('dark');
    } else {
        applyTheme('light');
    }

    // --- Auth Status ---
    fetch('/auth/me')
        .then(response => {
            if (response.ok) {
                return response.json();
            }
            throw new Error('Not logged in');
        })
        .then(user => {
            const userHtml = `
                <span>Welcome, ${user.username}</span>
                <a href="/auth/logout">Logout</a>
            `;
            userStatusDiv.insertAdjacentHTML('afterbegin', userHtml);
        })
        .catch(() => {
            const loginHtml = '<a href="/login.html">Login</a>';
            userStatusDiv.insertAdjacentHTML('afterbegin', loginHtml);
        });

    // --- Search Functionality ---
    const performSearch = () => {
        const query = searchInput.value;
        if (!query) {
            return;
        }

        const language = languageSelect.value;
        const engine = engineSelect.value;

        const searchUrl = `/search/ai?q=${encodeURIComponent(query)}&engine=${engine}&language=${language}`;
        window.location.href = searchUrl;
    };

    searchButton.addEventListener('click', performSearch);

    searchInput.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
            performSearch();
        }
    });
});
