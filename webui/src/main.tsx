import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import App from './components/App/App.tsx'
import ServiceDetail from './components/ServiceDetail/ServiceDetail.tsx'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path=":port" element={<ServiceDetail />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
)
