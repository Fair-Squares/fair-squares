'use client';

import { Sidebar } from 'flowbite-react';
import {
  MdDashboard,
  MdSignalWifiStatusbarConnectedNoInternet4,
  MdRoomService,
  MdSupervisorAccount,
} from 'react-icons/md';
import { Link, NavLink } from 'react-router-dom';

const logo = require('../../assets/android-chrome-192x192.png');
export default function SideBar() {
  return (
    <Sidebar
      aria-label="Sidebar with multi-level dropdown example"
      className="items-center bg-pink-700 text-white px-2"
    >
      <Sidebar.Items className="bg-pink-700 ">
        <Sidebar.ItemGroup>
          <Link to="./">
            <img src={logo} alt="FairSquares logo" width={100} height={100} />
          </Link>
        </Sidebar.ItemGroup>
        <Sidebar.ItemGroup>
          <Sidebar.Item labelColor="white" href="#" icon={MdDashboard} className="p-3">
            <NavLink to="dashboard">Dashboard</NavLink>
          </Sidebar.Item>
          <Sidebar.Collapse
            icon={MdSignalWifiStatusbarConnectedNoInternet4}
            label="Status"
            className="p-2"
          >
            <Sidebar.Item href="#">
              <NavLink to="roles">Roles</NavLink>
            </Sidebar.Item>
            <Sidebar.Item href="#">Assets</Sidebar.Item>
            <Sidebar.Item href="#">Referendums</Sidebar.Item>
          </Sidebar.Collapse>

          <Sidebar.Collapse icon={MdSupervisorAccount} label="Users" className="p-2">
            <Sidebar.Item href="#">Investors</Sidebar.Item>
            <Sidebar.Item href="#">Sellers</Sidebar.Item>
            <Sidebar.Item href="#">Tenants</Sidebar.Item>
          </Sidebar.Collapse>
          <Sidebar.Collapse icon={MdRoomService} label="Service Providers" className="p-2">
            <Sidebar.Item href="#">
            <NavLink to="council">Councils</NavLink>
            </Sidebar.Item>
            <Sidebar.Item href="#">Notaries</Sidebar.Item>
            <Sidebar.Item href="#">Representatives</Sidebar.Item>
          </Sidebar.Collapse>
        </Sidebar.ItemGroup>
      </Sidebar.Items>
    </Sidebar>
  );
}
