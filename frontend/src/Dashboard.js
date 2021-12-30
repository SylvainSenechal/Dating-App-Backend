import { useState, useEffect } from 'react';
import ImageUploader from './ImageUploader';

const Dashboard = ({ user, setUser }) => {
  console.log(user)
  const [date, setDate] = useState(Math.floor(Date.now() / 1000))
  const [refresh, setRefresh] = useState(0)

  const tokenData64URL = user.token.split('.')[1]
  const tokenB64 = tokenData64URL.replace(/-/g, '+').replace(/_/g, '/')
  const tokenPayload = JSON.parse(atob(tokenB64))
  const { pseudo, sub, iat, exp } = tokenPayload
console.log(tokenPayload)
  console.log("token payload :", pseudo, sub, iat, exp)

  useEffect(() => {
    const timer = setInterval(() => {
      setDate(Math.floor(Date.now() / 1000))
    }, 1000)
    return () => clearTimeout(timer);
  }, [])

  const logout = () => {
    window.localStorage.setItem('refreshToken', "")
    window.sessionStorage.setItem('refreshToken', "")
    setUser(prev => ({ ...prev, loggedIn: false, keepConnected: false, token: "", refreshToken: "" }))
  }

  return (
    <div id="dashboardOut">
      <div id="dashboardIn">
        <div id="infos" className="dashboardElement">
          <div> Hello {pseudo}, your id is {sub} </div>
          <div> Your token is valid for {exp - date} second{(exp - date) > 1 ? 's' : ''} </div>
        </div>
        <div id="logout" className="dashboardElement">
          <button onClick={logout}> Logout </button>
        </div>
        <ImageUploader token={user.token} />
      </div>
    </div>
  )
}

export default Dashboard;