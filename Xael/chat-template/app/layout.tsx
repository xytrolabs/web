import React, { ReactNode } from 'react';
import '@/styles/App.css';
import AppWrappers from './AppWrappers';

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body id="root">
        <AppWrappers>
          {children}
        </AppWrappers>
      </body>
    </html>
  );
}
