module.exports = {
    publicPath: process.env.TAURI_BUILD ? './' : '/',
    transpileDependencies: ['n-link-core']
}
