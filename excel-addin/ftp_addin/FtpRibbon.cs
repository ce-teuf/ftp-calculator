using System.Runtime.InteropServices;
using System.Windows.Forms;
using ExcelDna.Integration.CustomUI;

namespace FtpAddIn
{
    [ComVisible(true)]
    public class FtpRibbon : ExcelRibbon
    {
        public override string GetCustomUI(string ribbonID)
        {
            return @"
            <customUI xmlns='http://schemas.microsoft.com/office/2009/07/customui'>
              <ribbon>
                <tabs>
                  <tab id='ftpTab' label='FTP'>
                    <group id='ftpGroup' label='FTP Calculator'>
                      <button id='btnAbout'
                              label='About'
                              size='large'
                              imageMso='Info'
                              onAction='OnAbout' />
                    </group>
                  </tab>
                </tabs>
              </ribbon>
            </customUI>";
        }

        public void OnAbout(IRibbonControl control)
        {
            MessageBox.Show(
                "FTP Calculator Add-In\n\n" +
                "Powered by ftp_core (Rust) via Excel-DNA.\n" +
                "Use FTP_COMPUTE_STOCK / FTP_COMPUTE_FLUX to run calculations.",
                "About FTP Calculator",
                MessageBoxButtons.OK,
                MessageBoxIcon.Information);
        }
    }
}
