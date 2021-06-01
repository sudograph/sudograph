const Generator = require('yeoman-generator');

module.exports = class extends Generator {
    writing() {
        this.fs.copy(
            this.templatePath(),
            this.destinationPath(),
            {
                globOptions: {
                    ignore: this.templatePath('gitignore')
                }
            }
        );

        this.fs.copy(
            this.templatePath('gitignore'),
            this.destinationPath('.gitignore')
        );
    }
};