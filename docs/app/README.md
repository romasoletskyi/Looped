## Small guide for serving servers on github pages

Problem: If you do not register custom domain, page will be served from https. Thus, mixed content can't be served and all inner requests have to be made to https sites.

Solution:
1. Register at app.zerossl.com and start creating new certificate with IP of your server
2. Choose http file upload validation and set up needed folder and file stucture on server
3. Run ```sudo python3 -m http.server 80``` to serve folder contents and pass validation
4. Obtain certificate.crt and private.key
5. For this particular server, store them in the same folder as app
