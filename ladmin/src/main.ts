import './main.css'
// import 'virtual:uno.css'
import './load-indicators.css'
// import '/encre.css'

const BASE_PATH = import.meta.env.BASE_URL

import { main } from './ladmin.gleam'

main(BASE_PATH)
