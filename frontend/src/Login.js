import { useState } from 'react';
// import Footer from './Footer'

const Login = ({ setUser }) => {
  const [pseudoRegister, setPseudoRegister] = useState("My Pseudo")
  const [passwordRegister, setPasswordRegister] = useState("")
  const [pseudoLogin, setPseudoLogin] = useState("")
  const [passwordLogin, setPasswordLogin] = useState("")
  const [messageRegister, setMessageRegister] = useState("")
  const [keepConnected, setKeepConnected] = useState(false)

  const handleSubmitRegistration = async event => {
    event.preventDefault()
    console.log(0)

    const result = await fetch('http://localhost:8080/users', {
      method: 'POST', // *GET, POST, PUT, DELETE, etc.
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pseudo: pseudoRegister, password: passwordRegister })
    })
    console.log(1)
    const readableResult = await result.json()
    console.log(3)
    console.log(readableResult)

    setMessageRegister(readableResult.message)
  }

  const handleSubmitLogin = async event => {
    event.preventDefault()

    const result = await fetch('http://localhost:8080/auth', {
      method: 'POST', // *GET, POST, PUT, DELETE, etc.
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pseudo: pseudoLogin, password: passwordLogin })
    })
    console.log(result)

    const readableResult = await result.json()

    console.log(result)
    console.log("heeehe")
    console.log(readableResult)
    console.log(readableResult.status)
    if (result.status === 200) { // login successfull
      setUser(prev => ({ ...prev, token: readableResult.token, refreshToken: readableResult.refresh_token, loggedIn: true, keepConnected: keepConnected }))
      console.log("connexion")
      if (keepConnected) {
        console.log("keep")

        window.localStorage.setItem('refreshToken', readableResult.refresh_token)
      } else {
        console.log("dont keep")

        window.sessionStorage.setItem('refreshToken', readableResult.refresh_token)
      }
      console.log(window.localStorage)
      console.log(window.sessionStorage)
    }
  }

  return (
    <div className="LoginPage">
      <div className="formsLoginRegister" >
        <div className="logInfo" style={{ "--order": 0 }}>
          <p className="borderLine"> Register </p>
          <form onSubmit={handleSubmitRegistration}>
            <label htmlFor="pseudo"> Enter your name: </label>
            <input type="text" name="pseudo" id="pseudoRegister" value={pseudoRegister} onChange={e => setPseudoRegister(e.target.value)} required />
            <label htmlFor="password"> Enter your password: </label>
            <input type="password" name="password" id="passwordRegister" value={passwordRegister} onChange={e => setPasswordRegister(e.target.value)} required />
            <input type="submit" value="Register" />
          </form>
          <div> {messageRegister} </div>
        </div>

        <div className="logInfo" style={{ "--order": 1 }}>
          <p className="borderLine"> Login </p>
          <form onSubmit={handleSubmitLogin}>
            <label htmlFor="pseudo"> Enter your name: </label>
            <input type="text" name="pseudo" id="pseudoLoginh" value={pseudoLogin} onChange={e => setPseudoLogin(e.target.value)} required />
            <label htmlFor="password"> Enter your password: </label>
            <input type="password" name="password" id="passwordLogin" value={passwordLogin} onChange={e => setPasswordLogin(e.target.value)} required />
            <label htmlFor="keepConnectede"> Keep me connected :</label>
            <input
              name="keepConnected"
              type="checkbox"
              checked={keepConnected}
              onChange={e => setKeepConnected(e.target.checked)}
            />
            <input type="submit" value="Register" />
          </form>
        </div>
      </div>
      {/* <form action="http://localhost:8080/photos" method="post" encType="multipart/form-data">
        <input type="file" multiple name="file"/>
        <button type="submit">Submit</button>
      </form> */}
      {/* < Footer /> */}
    </div>
  )
}

export default Login;