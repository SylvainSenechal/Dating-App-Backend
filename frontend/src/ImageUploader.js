import { useState, useEffect } from 'react';


const ImageUploader = ({ token }) => {
    const [selectedFile, setSelectedFile] = useState();
    const [fileSelected, setFiledSelected] = useState(false);

    const handleChangeFile = event => {
        console.log(event)
        console.log(event.target)
        console.log(event.target.files)
        console.log(event.target.files[0])
        // TODO : handle errors + check type = jpeg ?
        setSelectedFile(event.target.files[0])
        setFiledSelected(true)
    }
  
    const handleSubmitFile = async event => {
    //   event.preventDefault()
  
   //   event.preventDefault()
   console.log("oiii")
   const formData = new FormData()
   formData.append('File', selectedFile) // todo : handle if file doesnt exist ?

   const result = await fetch('http://localhost:8080/photos', {
     method: 'POST', // *GET, POST, PUT, DELETE, etc.
     headers: { 'Authorization': `Bearer ${token}` },
     // body: JSON.stringify({ pseudo: pseudoRegister, password: passwordRegister })
     body: formData
   })
   console.log(1)
   const readableResult = await result.json()
   console.log(3)
   console.log(readableResult)
 
    }
  
    return (
    //     <form action="http://localhost:8080/photos" method="post" encType="multipart/form-data">
    //               <input type="file" multiple name="file"/>
    //               <button type="submit">Submit</button>
    //           </form>
    //     {/* < Footer /> */}
    //   </div>
    <div>
        <input type="file" name="file" onChange={handleChangeFile} />
        <div>
            <button onClick={handleSubmitFile}> Submit file </button>
        </div>
    </div>
    )
  }
  
  export default ImageUploader;