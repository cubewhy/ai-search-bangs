document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('search-input');
    const searchButton = document.getElementById('search-button');
    const languageSelect = document.getElementById('language-select');
    const engineSelect = document.getElementById('engine-select');
    const userStatusDiv = document.getElementById('user-status');

    fetch('/auth/me')
        .then(response => {
            if (response.ok) {
                return response.json();
            }
            throw new Error('Not logged in');
        })
        .then(user => {
            userStatusDiv.innerHTML = `
                <span>Welcome, ${user.username}</span>
                <a href="/auth/logout">Logout</a>
            `;
        })
        .catch(() => {
            userStatusDiv.innerHTML = '<a href="/login.html">Login</a>';
        });

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
