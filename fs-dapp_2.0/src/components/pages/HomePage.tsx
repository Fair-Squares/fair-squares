import React from 'react';
import { NavLink } from 'react-router-dom';

function HomePage() {
  const videoBg = require('../../assets/FairPage00.mp4');
  return (
    <div className="w-full h-screen">
      <video
        src={videoBg}
        autoPlay
        loop
        muted
        className="object-cover w-full h-screen overflow-clip "
      />
      <div className="absolute w-full h-screen flex  justify-center items-center top-20 backdrop-brightness-50">
        <span className="animate-pulse text-white ">
          <NavLink to="dashboard">
            <h2 className="font-bold -translate-x-20 text-7xl">Go to Dashboard</h2>
          </NavLink>
        </span>
      </div>
    </div>
  );
}

export default HomePage;
