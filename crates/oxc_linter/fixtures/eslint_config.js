module.exports = {
    rules: {
        // requires jsonc support
        'no-console': 'off',
        'no-bitwise': [
            'error',
            {
                allow: ['~'],
            },
        ],
        eqeqeq: [
            'error',
            'always',
            {
                null: 'ignore',
            },
        ],
        '@typescript-eslint/ban-types': 'error',
        'jsx-a11y/alt-text': 'warn',
    },
}
