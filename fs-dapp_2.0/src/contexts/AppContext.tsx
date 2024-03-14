import { createContext, useContext, useReducer, ReactNode } from 'react';
import { ApiPromise } from '@polkadot/api';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { AppState } from './types';
import BN from 'bn.js';

const initialState: AppState = {
  api: null,
  accounts: [],
  selectedAccount: undefined,
  selectedAddress: '',
  blocks: '',
  total_users_nbr: 0,
  inv_nbr: 0,
  seller_nbr: 0,
  awaiting_seller_nbr: 0,
  servicer_nbr: 0,
  awaiting_servicer_nbr: 0,
  tenant_nbr: 0,
  treasury_balance: undefined,
  web3Name: undefined,
  attester: undefined,
  credentials: undefined,
};

type Action =
  | { type: 'SET_API'; payload: ApiPromise | null }
  | { type: 'SET_ACCOUNTS'; payload: InjectedAccountWithMeta[] }
  | { type: 'SET_SELECTED_ACCOUNT'; payload: InjectedAccountWithMeta | undefined }
  | { type: 'SET_SELECTED_ADDRESS'; payload: string }
  | { type: 'SET_BLOCKS'; payload: string }
  | { type: 'SET_INVESTORS_NBR'; payload: number }
  | { type: 'SET_SELLERS_NBR'; payload: number }
  | { type: 'SET_A_SELLERS_NBR'; payload: number }
  | { type: 'SET_SERVICER_NBR'; payload: number }
  | { type: 'SET_A_SERVICER_NBR'; payload: number }
  | { type: 'SET_TENANTS_NBR'; payload: number }
  | { type: 'SET_TOTAL'; payload: number }
  | { type: 'SET_TREASURY_BALANCE'; payload: BN }
  | { type: 'SET_WEB3_NAME'; payload: string | undefined }
  | { type: 'SET_ATTESTER'; payload: string | undefined }
  | { type: 'SET_CREDENTIALS'; payload: string | undefined };

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case 'SET_API':
      return { ...state, api: action.payload };
    case 'SET_ACCOUNTS':
      return { ...state, accounts: action.payload };
    case 'SET_SELECTED_ACCOUNT':
      return { ...state, selectedAccount: action.payload };
    case 'SET_SELECTED_ADDRESS':
      return { ...state, selectedAddress: action.payload };
    case 'SET_BLOCKS':
      return { ...state, blocks: action.payload };
    case 'SET_INVESTORS_NBR':
      return { ...state, inv_nbr: action.payload };
    case 'SET_SELLERS_NBR':
      return { ...state, seller_nbr: action.payload };
    case 'SET_A_SELLERS_NBR':
      return { ...state, awaiting_seller_nbr: action.payload };

    case 'SET_SERVICER_NBR':
      return { ...state, seller_nbr: action.payload };
    case 'SET_A_SERVICER_NBR':
      return { ...state, awaiting_servicer_nbr: action.payload };
    case 'SET_TENANTS_NBR':
      return { ...state, tenant_nbr: action.payload };
    case 'SET_TOTAL':
      return { ...state, total_users_nbr: action.payload };
    case 'SET_TREASURY_BALANCE':
      return { ...state, treasury_balance: action.payload };
    case 'SET_WEB3_NAME':
      return { ...state, web3Name: action.payload };
    case 'SET_ATTESTER':
      return { ...state, attester: action.payload };
    case 'SET_CREDENTIALS':
      return { ...state, credentials: action.payload };

    default:
      return state;
  }
}

type AppContextType = AppState & {
  dispatch: React.Dispatch<Action>;
};
const AppContext = createContext<AppContextType>({
  ...initialState,
  dispatch: () => {},
});

type Props = {
  children: ReactNode;
};
export function AppProvider({ children }: Props) {
  const [
    {
      api,
      accounts,
      selectedAccount,
      selectedAddress,
      blocks,
      total_users_nbr,
      inv_nbr,
      seller_nbr,
      awaiting_seller_nbr,
      servicer_nbr,
      awaiting_servicer_nbr,
      tenant_nbr,
      treasury_balance,
      web3Name,
      attester,
      credentials,
    },
    dispatch,
  ] = useReducer(reducer, initialState);
  return (
    <AppContext.Provider
      value={{
        api,
        accounts,
        selectedAccount,
        selectedAddress,
        blocks,
        total_users_nbr,
        inv_nbr,
        seller_nbr,
        awaiting_seller_nbr,
        servicer_nbr,
        awaiting_servicer_nbr,
        tenant_nbr,
        treasury_balance,
        web3Name,
        attester,
        credentials,
        dispatch,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}
export const useAppContext = () => useContext(AppContext);
