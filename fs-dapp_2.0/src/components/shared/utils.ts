import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import BN from 'bn.js';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
export function toUnit(balance: BN, decimals: number) {
  let base = new BN(10).pow(new BN(decimals));
  let dm = new BN(balance).divmod(base);
  return parseFloat(dm.div.toString() + '.' + dm.mod.toString());
}
