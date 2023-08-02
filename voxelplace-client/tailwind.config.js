module.exports = {
    theme: {
        extend: {
            fontFamily: {
                'roboto': ['Roboto', 'sans-serif'],
            },
            boxShadow: {
                custom: '8px 8px 0px 0px rgba(0, 0, 0, 0.25)',
            },
            transitionProperty: {
                'width': 'width',
                'spacing': 'margin, padding',
            }
        }
    },
    content: [
        './pages/**/*.vue',
        './layouts/**/*.vue',
        './components/**/*.vue',
    ],
}
