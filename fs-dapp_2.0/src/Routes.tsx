import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import App from './App';
import Dashboard from './components/pages/Dashboard';
import HomePage from './components/pages/HomePage';
import Roles from './components/pages/Roles';
import Council from './components/pages/Council';

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'dashboard', element: <Dashboard /> },
      { path: 'roles', element: <Roles /> },
      { path: 'council', element: <Council />},
    ],
  },
]);
export function Routes() {
  return <RouterProvider router={router} />;
}
