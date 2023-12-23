import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

export default defineConfig({
    plugins: [react()],
    publicDir: 'public',
    css: {
        modules: {
            localsConvention: 'camelCaseOnly',
        },
    },
    json: {
        namedExports: true,
    },
    esbuild: {
        jsxInject: `import React from 'react'`,
    },
    logLevel: 'info',
})
