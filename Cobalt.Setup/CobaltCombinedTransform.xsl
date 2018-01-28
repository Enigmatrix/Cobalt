<?xml version="1.0" ?>
<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:wix="http://schemas.microsoft.com/wix/2006/wi">

  <!-- Copy all attributes and elements to the output. -->
  <xsl:template match="@*|*">
    <xsl:copy>
      <xsl:apply-templates select="@*" />
      <xsl:apply-templates select="*" />
    </xsl:copy>
  </xsl:template>

  <xsl:output method="xml" indent="yes" />


  <xsl:key name="dll-search" match="wix:Component[not(contains(wix:File/@Source, '.dll') or contains(wix:File/@Source, '.exe'))]" use="@Id" />
  <xsl:template match="wix:Component[key('dll-search', @Id)]" />
  <xsl:template match="wix:ComponentRef[key('dll-search', @Id)]" />

</xsl:stylesheet>