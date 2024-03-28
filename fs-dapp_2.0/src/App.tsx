import React from 'react';
import { AppProvider } from './contexts/AppContext';
import { AccountProvider } from './contexts/Account_Context';
import { CouncilSessionProvider } from './contexts/CouncilSessionContext';
import { Layout } from './components/shared/Layout';

function App() {
  return (
    <div className="app">
      <AppProvider>
        <AccountProvider>
          <CouncilSessionProvider>
            <Layout />
          </CouncilSessionProvider>
        </AccountProvider>
      </AppProvider>
    </div>
  );
}

export default App;
